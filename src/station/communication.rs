use crate::map::Tile;
use crate::robot::Position;
use crate::robot::resources::ResourceType;
use crossbeam::channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RobotMessage {
    ResourceDiscovered {
        resource_type: ResourceType,
        position: Position,
    },
    ResourceConsumed {
        resource_type: ResourceType,
        position: Position,
        amount: u32,
        robot_id: u32,
    },
    RequestResourcesState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StationMessage {
    ResourcesUpdate {
        energy_resources: HashMap<Position, u32>,
        mineral_resources: HashMap<Position, u32>,
        scientific_resources: HashMap<Position, u32>,
    },
    ResourceUpdate {
        resource_type: ResourceType,
        position: Position,
        remaining: u32,
    },
    Acknowledgement {
        message: String,
    },
}

pub struct StationCommunication {
    pub robot_receiver: Receiver<(u32, RobotMessage)>,
    pub robot_senders: HashMap<u32, Sender<StationMessage>>,
    energy_resources: HashMap<Position, u32>,
    mineral_resources: HashMap<Position, u32>,
    scientific_resources: HashMap<Position, u32>,
}

// / Implémentation de la communication entre la station et les robots
impl StationCommunication {
    pub fn new(robot_receiver: Receiver<(u32, RobotMessage)>) -> Self {
        Self {
            robot_receiver,
            robot_senders: HashMap::new(),
            energy_resources: HashMap::new(),
            mineral_resources: HashMap::new(),
            scientific_resources: HashMap::new(),
        }
    }

    pub fn register_robot(&mut self, robot_id: u32, sender: Sender<StationMessage>) {
        self.robot_senders.insert(robot_id, sender);
    }

    pub fn process_messages(&mut self, map: &mut crate::map::Map) {
        while let Ok((robot_id, message)) = self.robot_receiver.try_recv() {
            match message {
                RobotMessage::ResourceDiscovered {
                    resource_type,
                    position,
                } => {
                    if let Some(tile) = map.get_tile(position.x, position.y) {
                        match (resource_type, tile) {
                            (ResourceType::Energy, Tile::Energy(energy)) => {
                                self.energy_resources
                                    .insert(position, energy.amount);
                            }
                            (ResourceType::Minerals, Tile::Mineral(mineral)) => {
                                self.mineral_resources
                                    .insert(position, mineral.amount);
                            }
                            (ResourceType::ScientificData, Tile::ScientificPoint(point)) => {
                                self.scientific_resources
                                    .insert(position, point.value);
                            }
                            _ => {}
                        }
                    }
    
                    if let Some(sender) = self.robot_senders.get(&robot_id) {
                        let _ = sender.send(StationMessage::Acknowledgement {
                            message: format!("Resource discovered at {:?} registered", position),
                        });
                    }
                }
                RobotMessage::ResourceConsumed {
                    resource_type,
                    position,
                    amount,
                    robot_id: _,
                } => {
                    // Mettre à jour la carte et l'état des ressources
                    match resource_type {
                        ResourceType::Energy => {
                            let mut should_remove = false;
                            let mut new_amount = 0;
                            
                            if let Some(current) = self.energy_resources.get(&position) {
                                new_amount = current.saturating_sub(amount);
                                should_remove = new_amount == 0;
                            }
                            
                            if should_remove {
                                self.energy_resources.remove(&position);
                                new_amount = 0;
                            } else if new_amount > 0 {
                                self.energy_resources.insert(position, new_amount);
                            }
                            
                            self.broadcast_resource_update(resource_type, position, new_amount);
                            
                            map.consume_energy(position.x, position.y, amount);
                        }
                        ResourceType::Minerals => {
                            let mut should_remove = false;
                            let mut new_amount = 0;
                            
                            if let Some(current) = self.mineral_resources.get(&position) {
                                new_amount = current.saturating_sub(amount);
                                should_remove = new_amount == 0;
                            }
                            
                            if should_remove {
                                self.mineral_resources.remove(&position);
                                new_amount = 0;
                            } else if new_amount > 0 {
                                self.mineral_resources.insert(position, new_amount);
                            }
                            
                            self.broadcast_resource_update(resource_type, position, new_amount);
                            
                            map.consume_mineral(position.x, position.y, amount);
                        }
                        ResourceType::ScientificData => {
                            let exists = self.scientific_resources.contains_key(&position);
                            
                            if exists {
                                self.scientific_resources.remove(&position);
                                self.broadcast_resource_update(resource_type, position, 0);
                            }
                            
                            map.extract_scientific_data(position.x, position.y);
                        }
                    }
    
                    if let Some(sender) = self.robot_senders.get(&robot_id) {
                        let _ = sender.send(StationMessage::Acknowledgement {
                            message: format!("Resource consumption at {:?} registered", position),
                        });
                    }
                }
                // Envoyer l'état complet des ressources au robot qui le demande
                RobotMessage::RequestResourcesState => {
                    if let Some(sender) = self.robot_senders.get(&robot_id) {
                        let _ = sender.send(StationMessage::ResourcesUpdate {
                            energy_resources: self.energy_resources.clone(),
                            mineral_resources: self.mineral_resources.clone(),
                            scientific_resources: self.scientific_resources.clone(),
                        });
                    }
                }
            }
        }
    }

    fn broadcast_resource_update(&self, resource_type: ResourceType, position: Position, remaining: u32) {
        let update = StationMessage::ResourceUpdate {
            resource_type,
            position,
            remaining,
        };

        for sender in self.robot_senders.values() {
            let _ = sender.send(update.clone());
        }
    }

    pub fn broadcast_full_state(&self) {
        let update = StationMessage::ResourcesUpdate {
            energy_resources: self.energy_resources.clone(),
            mineral_resources: self.mineral_resources.clone(),
            scientific_resources: self.scientific_resources.clone(),
        };

        for sender in self.robot_senders.values() {
            let _ = sender.send(update.clone());
        }
    }

    pub fn get_resources_state(&self) -> (HashMap<Position, u32>, HashMap<Position, u32>, HashMap<Position, u32>) {
        (
            self.energy_resources.clone(),
            self.mineral_resources.clone(),
            self.scientific_resources.clone(),
        )
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{Map, Energy, Mineral, ScientificPoint};
    use crate::robot::resources::ResourceType;
    use crossbeam::channel;
    use std::thread;
    use std::time::Duration;

    fn create_test_map() -> Map {
        let mut map = Map::new(100, 100, 12345);
        
        if let Some(tile) = map.get_tile_mut(10, 10) {
            *tile = crate::map::Tile::Energy(Energy { amount: 1000, is_base: true });
        }
        
        if let Some(tile) = map.get_tile_mut(20, 20) {
            *tile = crate::map::Tile::Mineral(Mineral { amount: 500, is_base: true });
        }
        
        if let Some(tile) = map.get_tile_mut(30, 30) {
            *tile = crate::map::Tile::ScientificPoint(ScientificPoint { value: 300, is_base: true });
        }
        
        map
    }

    #[test]
    fn test_register_robot() {
        let (sender, receiver) = channel::unbounded();
        let mut comm = StationCommunication::new(receiver);
        
        let (robot_sender, _) = channel::unbounded();
        comm.register_robot(1, robot_sender.clone());
        
        assert!(comm.robot_senders.contains_key(&1));
        assert_eq!(comm.robot_senders.len(), 1);
        
        let (robot_sender2, _) = channel::unbounded();
        comm.register_robot(2, robot_sender2);
        
        assert!(comm.robot_senders.contains_key(&2));
        assert_eq!(comm.robot_senders.len(), 2);
    }
    
    #[test]
    fn test_resource_discovery() {
        let (sender, receiver) = channel::unbounded();
        let mut comm = StationCommunication::new(receiver);
        let mut map = create_test_map();
        
        let (robot_sender, robot_receiver) = channel::unbounded();
        comm.register_robot(1, robot_sender);
        
        let position = Position { x: 10, y: 10 };
        let _ = sender.send((1, RobotMessage::ResourceDiscovered { 
            resource_type: ResourceType::Energy, 
            position 
        }));
        
        comm.process_messages(&mut map);
        
        let (energy, _, _) = comm.get_resources_state();
        assert!(energy.contains_key(&position));
        assert_eq!(energy.get(&position), Some(&1000));
        
        if let Ok(message) = robot_receiver.recv_timeout(Duration::from_millis(100)) {
            match message {
                StationMessage::Acknowledgement { message: _ } => (),
                _ => panic!("Type de message incorrect reçu"),
            }
        } else {
            panic!("Aucun message de confirmation reçu");
        }
    }
    
    #[test]
    fn test_resource_consumption() {
        let (sender, receiver) = channel::unbounded();
        let mut comm = StationCommunication::new(receiver);
        let mut map = create_test_map();
        
        let (robot1_sender, _) = channel::unbounded();
        let (robot2_sender, robot2_receiver) = channel::unbounded();
        comm.register_robot(1, robot1_sender);
        comm.register_robot(2, robot2_sender);
        
        let position = Position { x: 10, y: 10 };
        let _ = sender.send((1, RobotMessage::ResourceDiscovered { 
            resource_type: ResourceType::Energy, 
            position 
        }));
        
        comm.process_messages(&mut map);
        
        let _ = sender.send((1, RobotMessage::ResourceConsumed { 
            resource_type: ResourceType::Energy, 
            position,
            amount: 200,
            robot_id: 1 
        }));
        
        comm.process_messages(&mut map);
        
        let (energy, _, _) = comm.get_resources_state();
        assert_eq!(energy.get(&position), Some(&800)); 
        
        if let Ok(message) = robot2_receiver.recv_timeout(Duration::from_millis(100)) {
            match message {
                StationMessage::ResourceUpdate { 
                    resource_type, 
                    position: pos, 
                    remaining 
                } => {
                    assert_eq!(resource_type, ResourceType::Energy);
                    assert_eq!(pos, position);
                    assert_eq!(remaining, 800);
                },
                _ => panic!("Type de message incorrect reçu"),
            }
        } else {
            panic!("Aucun message de mise à jour reçu");
        }
    }
    
    #[test]
    fn test_request_resources_state() {
        let (sender, receiver) = channel::unbounded();
        let mut comm = StationCommunication::new(receiver);
        let mut map = create_test_map();
        
        let (robot_sender, robot_receiver) = channel::unbounded();
        comm.register_robot(1, robot_sender);
        
        let energy_pos = Position { x: 10, y: 10 };
        let mineral_pos = Position { x: 20, y: 20 };
        
        let _ = sender.send((1, RobotMessage::ResourceDiscovered { 
            resource_type: ResourceType::Energy, 
            position: energy_pos
        }));
        let _ = sender.send((1, RobotMessage::ResourceDiscovered { 
            resource_type: ResourceType::Minerals, 
            position: mineral_pos
        }));
        
        comm.process_messages(&mut map);
        
        while let Ok(message) = robot_receiver.try_recv() {
            match message {
                StationMessage::Acknowledgement { .. } => {
                    // Message de confirmation attendu, continuer
                },
                _ => {
                    panic!("Message inattendu reçu avant la demande d'état");
                }
            }
        }
        
        let _ = sender.send((1, RobotMessage::RequestResourcesState));
        comm.process_messages(&mut map);
        
        if let Ok(message) = robot_receiver.recv_timeout(Duration::from_millis(100)) {
            match message {
                StationMessage::ResourcesUpdate { 
                    energy_resources, 
                    mineral_resources, 
                    scientific_resources 
                } => {
                    assert!(energy_resources.contains_key(&energy_pos));
                    assert!(mineral_resources.contains_key(&mineral_pos));
                    assert_eq!(scientific_resources.len(), 0);
                },
                _ => panic!("Type de message incorrect reçu pour la mise à jour des ressources"),
            }
        } else {
            panic!("Aucun message d'état des ressources reçu");
        }
    }
}