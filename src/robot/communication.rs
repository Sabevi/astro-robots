use crate::robot::{Position, ResourceType};
use crate::station::communication::{RobotMessage, StationMessage};
use crossbeam::channel::{Receiver, Sender};
use std::collections::HashMap;

pub struct RobotCommunication {
    pub robot_id: u32,
    pub station_sender: Sender<(u32, RobotMessage)>,
    pub station_receiver: Receiver<StationMessage>,
    local_energy_resources: HashMap<Position, u32>,
    local_mineral_resources: HashMap<Position, u32>,
    local_scientific_resources: HashMap<Position, u32>,
    pending_consumed_resources: Vec<(ResourceType, Position, u32)>,
}

impl RobotCommunication {
    pub fn new(
        robot_id: u32,
        station_sender: Sender<(u32, RobotMessage)>,
        station_receiver: Receiver<StationMessage>,
    ) -> Self {
        Self {
            robot_id,
            station_sender,
            station_receiver,
            local_energy_resources: HashMap::new(),
            local_mineral_resources: HashMap::new(),
            local_scientific_resources: HashMap::new(),
            pending_consumed_resources: Vec::new(),
        }
    }

    pub fn report_resource_discovered(&self, resource_type: ResourceType, position: Position) {
        let message = RobotMessage::ResourceDiscovered {
            resource_type,
            position,
        };
        let _ = self.station_sender.send((self.robot_id, message));
    }

    pub fn register_consumed_resource(
        &mut self,
        resource_type: ResourceType,
        position: Position,
        amount: u32,
    ) {
        match resource_type {
            ResourceType::Energy => {
                if let Some(current) = self.local_energy_resources.get_mut(&position) {
                    *current = current.saturating_sub(amount);
                    if *current == 0 {
                        self.local_energy_resources.remove(&position);
                    }
                }
            }
            ResourceType::Minerals => {
                if let Some(current) = self.local_mineral_resources.get_mut(&position) {
                    *current = current.saturating_sub(amount);
                    if *current == 0 {
                        self.local_mineral_resources.remove(&position);
                    }
                }
            }
            ResourceType::ScientificData => {
                self.local_scientific_resources.remove(&position);
            }
        }

        self.pending_consumed_resources
            .push((resource_type, position, amount));
    }

    pub fn report_pending_consumed_resources(&mut self) {
        for (resource_type, position, amount) in self.pending_consumed_resources.drain(..) {
            let message = RobotMessage::ResourceConsumed {
                resource_type,
                position,
                amount,
                robot_id: self.robot_id,
            };
            let _ = self.station_sender.send((self.robot_id, message));
        }
    }

    pub fn request_resources_state(&self) {
        let message = RobotMessage::RequestResourcesState;
        let _ = self.station_sender.send((self.robot_id, message));
    }

    pub fn process_station_messages(&mut self) {
        while let Ok(message) = self.station_receiver.try_recv() {
            match message {
                StationMessage::ResourcesUpdate {
                    energy_resources,
                    mineral_resources,
                    scientific_resources,
                } => {
                    self.local_energy_resources = energy_resources;
                    self.local_mineral_resources = mineral_resources;
                    self.local_scientific_resources = scientific_resources;
                }
                StationMessage::ResourceUpdate {
                    resource_type,
                    position,
                    remaining,
                } => {
                    match resource_type {
                        ResourceType::Energy => {
                            if remaining > 0 {
                                self.local_energy_resources.insert(position, remaining);
                            } else {
                                self.local_energy_resources.remove(&position);
                            }
                        }
                        ResourceType::Minerals => {
                            if remaining > 0 {
                                self.local_mineral_resources.insert(position, remaining);
                            } else {
                                self.local_mineral_resources.remove(&position);
                            }
                        }
                        ResourceType::ScientificData => {
                            if remaining > 0 {
                                self.local_scientific_resources.insert(position, remaining);
                            } else {
                                self.local_scientific_resources.remove(&position);
                            }
                        }
                    }
                }
                StationMessage::Acknowledgement { message: _ } => {
                    // Confirmer une action (pourrait être utilisé pour la log)
                }
            }
        }
    }

    pub fn is_resource_available(
        &self,
        resource_type: ResourceType,
        position: Position,
        required_amount: u32,
    ) -> bool {
        match resource_type {
            ResourceType::Energy => {
                if let Some(&amount) = self.local_energy_resources.get(&position) {
                    amount >= required_amount
                } else {
                    false
                }
            }
            ResourceType::Minerals => {
                if let Some(&amount) = self.local_mineral_resources.get(&position) {
                    amount >= required_amount
                } else {
                    false
                }
            }
            ResourceType::ScientificData => self.local_scientific_resources.contains_key(&position),
        }
    }

    pub fn get_local_resources_state(
        &self,
    ) -> (
        &HashMap<Position, u32>,
        &HashMap<Position, u32>,
        &HashMap<Position, u32>,
    ) {
        (
            &self.local_energy_resources,
            &self.local_mineral_resources,
            &self.local_scientific_resources,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::robot::resources::ResourceType;
    use crossbeam::channel;
    use std::time::Duration;

    #[test]
    fn test_robot_communication_init() {
        let (station_sender, _) = channel::unbounded();
        let (_, station_receiver) = channel::unbounded();
        
        let robot_comm = RobotCommunication::new(1, station_sender, station_receiver);
        
        assert_eq!(robot_comm.robot_id, 1);
        assert_eq!(robot_comm.local_energy_resources.len(), 0);
        assert_eq!(robot_comm.local_mineral_resources.len(), 0);
        assert_eq!(robot_comm.local_scientific_resources.len(), 0);
        assert_eq!(robot_comm.pending_consumed_resources.len(), 0);
    }
    
    #[test]
    fn test_report_resource_discovered() {
        let (station_sender, station_receiver) = channel::unbounded();
        let (_, robot_receiver) = channel::unbounded();
        
        let robot_comm = RobotCommunication::new(1, station_sender, robot_receiver);
        
        let position = Position { x: 15, y: 15 };
        robot_comm.report_resource_discovered(ResourceType::Energy, position);
        
        if let Ok((robot_id, message)) = station_receiver.recv_timeout(Duration::from_millis(100)) {
            assert_eq!(robot_id, 1);
            match message {
                RobotMessage::ResourceDiscovered { 
                    resource_type, 
                    position: pos 
                } => {
                    assert_eq!(resource_type, ResourceType::Energy);
                    assert_eq!(pos, position);
                },
                _ => panic!("Type de message incorrect envoyé"),
            }
        } else {
            panic!("Aucun message envoyé à la station");
        }
    }
    
    #[test]
    fn test_register_consumed_resource() {
        let (station_sender, _) = channel::unbounded();
        let (_, station_receiver) = channel::unbounded();
        
        let mut robot_comm = RobotCommunication::new(1, station_sender, station_receiver);
        
        let position = Position { x: 25, y: 25 };
        robot_comm.local_energy_resources.insert(position, 500);
        
        robot_comm.register_consumed_resource(ResourceType::Energy, position, 100);
        
        assert_eq!(robot_comm.local_energy_resources.get(&position), Some(&400));
        
        assert_eq!(robot_comm.pending_consumed_resources.len(), 1);
        assert_eq!(robot_comm.pending_consumed_resources[0], 
                   (ResourceType::Energy, position, 100));
    }
    
    #[test]
    fn test_report_pending_consumed_resources() {
        let (station_sender, station_receiver) = channel::unbounded();
        let (_, robot_receiver) = channel::unbounded();
        
        let mut robot_comm = RobotCommunication::new(1, station_sender, robot_receiver);
        
        let position1 = Position { x: 5, y: 5 };
        let position2 = Position { x: 15, y: 15 };
        
        robot_comm.pending_consumed_resources.push((ResourceType::Energy, position1, 50));
        robot_comm.pending_consumed_resources.push((ResourceType::Minerals, position2, 30));
        
        robot_comm.report_pending_consumed_resources();
        
        assert_eq!(robot_comm.pending_consumed_resources.len(), 0);
        
        for _ in 0..2 {
            if let Ok((robot_id, message)) = station_receiver.recv_timeout(Duration::from_millis(100)) {
                assert_eq!(robot_id, 1);
                match message {
                    RobotMessage::ResourceConsumed { .. } => (),
                    _ => panic!("Type de message incorrect envoyé"),
                }
            } else {
                panic!("Un message n'a pas été envoyé à la station");
            }
        }
    }
    
    #[test]
    fn test_process_station_messages() {
        let (station_sender, _) = channel::unbounded();
        let (robot_sender, robot_receiver) = channel::unbounded();
        
        let mut robot_comm = RobotCommunication::new(1, station_sender, robot_receiver);
        
        let position = Position { x: 30, y: 30 };
        let _ = robot_sender.send(StationMessage::ResourceUpdate {
            resource_type: ResourceType::Energy,
            position,
            remaining: 800,
        });
        
        robot_comm.process_station_messages();
        
        assert_eq!(robot_comm.local_energy_resources.get(&position), Some(&800));
        
        let mut energy_map = HashMap::new();
        energy_map.insert(position, 750);
        let mineral_map = HashMap::new();
        let scientific_map = HashMap::new();
        
        let _ = robot_sender.send(StationMessage::ResourcesUpdate {
            energy_resources: energy_map.clone(),
            mineral_resources: mineral_map.clone(),
            scientific_resources: scientific_map.clone(),
        });
        
        robot_comm.process_station_messages();
        
        assert_eq!(robot_comm.local_energy_resources, energy_map);
        assert_eq!(robot_comm.local_mineral_resources, mineral_map);
        assert_eq!(robot_comm.local_scientific_resources, scientific_map);
    }
    
    #[test]
    fn test_is_resource_available() {
        let (station_sender, _) = channel::unbounded();
        let (_, station_receiver) = channel::unbounded();
        
        let mut robot_comm = RobotCommunication::new(1, station_sender, station_receiver);
        
        let energy_pos = Position { x: 40, y: 40 };
        let mineral_pos = Position { x: 50, y: 50 };
        let science_pos = Position { x: 60, y: 60 };
        
        robot_comm.local_energy_resources.insert(energy_pos, 200);
        robot_comm.local_mineral_resources.insert(mineral_pos, 100);
        robot_comm.local_scientific_resources.insert(science_pos, 1);
        
        // Tester la disponibilité des ressources
        assert!(robot_comm.is_resource_available(ResourceType::Energy, energy_pos, 100));
        assert!(!robot_comm.is_resource_available(ResourceType::Energy, energy_pos, 300));
        assert!(robot_comm.is_resource_available(ResourceType::Minerals, mineral_pos, 100));
        assert!(robot_comm.is_resource_available(ResourceType::ScientificData, science_pos, 1));
        assert!(!robot_comm.is_resource_available(ResourceType::Energy, Position { x: 999, y: 999 }, 10));
    }
}