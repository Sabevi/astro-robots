use crate::map::Tile;
use crate::robot::Position;
use crate::robot::resources::ResourceType;
use crossbeam::channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RobotMessage {
    /// Signale qu'une ressource a été découverte
    ResourceDiscovered {
        resource_type: ResourceType,
        position: Position,
    },
    /// Signale qu'une ressource a été consommée
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
    /// La clé est l'ID du robot, la valeur est l'émetteur vers ce robot
    pub robot_senders: HashMap<u32, Sender<StationMessage>>,
    energy_resources: HashMap<Position, u32>,
    mineral_resources: HashMap<Position, u32>,
    scientific_resources: HashMap<Position, u32>,
}

impl StationCommunication {
    /// Crée une nouvelle instance de la communication de la station
    pub fn new(robot_receiver: Receiver<(u32, RobotMessage)>) -> Self {
        Self {
            robot_receiver,
            robot_senders: HashMap::new(),
            energy_resources: HashMap::new(),
            mineral_resources: HashMap::new(),
            scientific_resources: HashMap::new(),
        }
    }

    /// Ajoute un robot au système de communication
    pub fn register_robot(&mut self, robot_id: u32, sender: Sender<StationMessage>) {
        self.robot_senders.insert(robot_id, sender);
    }

    pub fn process_messages(&mut self, map: &mut crate::map::Map) {
        // Traiter tous les messages disponibles
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
    
                    // Envoyer une confirmation au robot
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
                            // Obtenir la valeur actuelle et calculer la nouvelle valeur
                            let mut should_remove = false;
                            let mut new_amount = 0;
                            
                            if let Some(current) = self.energy_resources.get(&position) {
                                new_amount = current.saturating_sub(amount);
                                should_remove = new_amount == 0;
                            }
                            
                            // Mettre à jour ou supprimer selon le cas
                            if should_remove {
                                self.energy_resources.remove(&position);
                                new_amount = 0;
                            } else if new_amount > 0 {
                                self.energy_resources.insert(position, new_amount);
                            }
                            
                            // Diffuser la mise à jour
                            self.broadcast_resource_update(resource_type, position, new_amount);
                            
                            // Mettre à jour la carte réelle
                            map.consume_energy(position.x, position.y, amount);
                        }
                        ResourceType::Minerals => {
                            // Même logique pour les minéraux
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
                            // Les données scientifiques sont toujours entièrement consommées
                            let exists = self.scientific_resources.contains_key(&position);
                            
                            if exists {
                                self.scientific_resources.remove(&position);
                                self.broadcast_resource_update(resource_type, position, 0);
                            }
                            
                            map.extract_scientific_data(position.x, position.y);
                        }
                    }
    
                    // Envoyer une confirmation au robot
                    if let Some(sender) = self.robot_senders.get(&robot_id) {
                        let _ = sender.send(StationMessage::Acknowledgement {
                            message: format!("Resource consumption at {:?} registered", position),
                        });
                    }
                }
                RobotMessage::RequestResourcesState => {
                    // Envoyer l'état complet des ressources au robot qui le demande
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
        
        // Placer des ressources à des positions spécifiques pour les tests
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
        
        // Enregistrer un robot
        let (robot_sender, _) = channel::unbounded();
        comm.register_robot(1, robot_sender.clone());
        
        // Vérifier que le robot a été enregistré
        assert!(comm.robot_senders.contains_key(&1));
        assert_eq!(comm.robot_senders.len(), 1);
        
        // Enregistrer un deuxième robot
        let (robot_sender2, _) = channel::unbounded();
        comm.register_robot(2, robot_sender2);
        
        // Vérifier l'enregistrement
        assert!(comm.robot_senders.contains_key(&2));
        assert_eq!(comm.robot_senders.len(), 2);
    }
    
    #[test]
    fn test_resource_discovery() {
        let (sender, receiver) = channel::unbounded();
        let mut comm = StationCommunication::new(receiver);
        let mut map = create_test_map();
        
        // Enregistrer un robot et préparer son receveur
        let (robot_sender, robot_receiver) = channel::unbounded();
        comm.register_robot(1, robot_sender);
        
        // Simuler un robot découvrant une ressource
        let position = Position { x: 10, y: 10 };
        let _ = sender.send((1, RobotMessage::ResourceDiscovered { 
            resource_type: ResourceType::Energy, 
            position 
        }));
        
        // Traiter le message
        comm.process_messages(&mut map);
        
        // Vérifier que la ressource a été enregistrée
        let (energy, _, _) = comm.get_resources_state();
        assert!(energy.contains_key(&position));
        assert_eq!(energy.get(&position), Some(&1000));
        
        // Vérifier qu'une confirmation a été envoyée au robot
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
        
        // Enregistrer deux robots
        let (robot1_sender, _) = channel::unbounded();
        let (robot2_sender, robot2_receiver) = channel::unbounded();
        comm.register_robot(1, robot1_sender);
        comm.register_robot(2, robot2_sender);
        
        // Simuler la découverte d'une ressource
        let position = Position { x: 10, y: 10 };
        let _ = sender.send((1, RobotMessage::ResourceDiscovered { 
            resource_type: ResourceType::Energy, 
            position 
        }));
        
        // Traiter le message de découverte
        comm.process_messages(&mut map);
        
        // Simuler la consommation de cette ressource
        let _ = sender.send((1, RobotMessage::ResourceConsumed { 
            resource_type: ResourceType::Energy, 
            position,
            amount: 200,
            robot_id: 1 
        }));
        
        // Traiter le message de consommation
        comm.process_messages(&mut map);
        
        // Vérifier que la ressource a été mise à jour
        let (energy, _, _) = comm.get_resources_state();
        assert_eq!(energy.get(&position), Some(&800)); // 1000 - 200 = 800
        
        // Vérifier que robot2 a reçu une mise à jour
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
        
        // Enregistrer un robot
        let (robot_sender, robot_receiver) = channel::unbounded();
        comm.register_robot(1, robot_sender);
        
        // Découvrir quelques ressources
        let energy_pos = Position { x: 10, y: 10 };
        let mineral_pos = Position { x: 20, y: 20 };
        
        // Les ajouter à la connaissance de la station
        let _ = sender.send((1, RobotMessage::ResourceDiscovered { 
            resource_type: ResourceType::Energy, 
            position: energy_pos
        }));
        let _ = sender.send((1, RobotMessage::ResourceDiscovered { 
            resource_type: ResourceType::Minerals, 
            position: mineral_pos
        }));
        
        // Traiter les découvertes
        comm.process_messages(&mut map);
        
        // Le robot a dû recevoir des confirmations - les ignorer
        while let Ok(message) = robot_receiver.try_recv() {
            match message {
                StationMessage::Acknowledgement { .. } => {
                    // Message de confirmation attendu, continuer
                },
                _ => {
                    // Autre type de message, pas attendu à ce stade
                    panic!("Message inattendu reçu avant la demande d'état");
                }
            }
        }
        
        // Demander l'état des ressources
        let _ = sender.send((1, RobotMessage::RequestResourcesState));
        comm.process_messages(&mut map);
        
        // Vérifier que le robot a reçu un état complet
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