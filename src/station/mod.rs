#[allow(dead_code)]
pub mod communication;
pub mod production;
pub mod resources;
pub mod sync;

use crate::map::{Map, Tile};
use crate::robot::resources::ResourceType;
use crate::robot::{HardwareModule, Position, Resources, Robot};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiscoveredResources {
    pub energy_locations: Vec<Position>,
    pub mineral_locations: Vec<Position>,
    pub scientific_locations: Vec<Position>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub position: Position,
    pub resources: Resources,
    pub known_map: Option<Map>,
    pub robots: Vec<Robot>,
    pub max_robots: usize,
    pub production_costs: ProductionCosts,
    pub discovered_resources: DiscoveredResources,
}

fn find_nearby_empty_position(map: &Map, center: Position) -> Position {
    let max_radius = map.width.max(map.height) as i32;
    for radius in 0..max_radius {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs() != radius && dy.abs() != radius {
                    continue;
                }

                let x = center.x as i32 + dx;
                let y = center.y as i32 + dy;

                if x >= 0 && y >= 0 && (x as u32) < map.width && (y as u32) < map.height {
                    if let Some(Tile::Empty) = map.get_tile(x as u32, y as u32) {
                        return Position {
                            x: x as u32,
                            y: y as u32,
                        };
                    }
                }
            }
        }
    }

    Position { x: 0, y: 0 }
}

fn clear_area_around_station(map: &mut Map, station_pos: &Position) {
    let radius = 3; 

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = (station_pos.x as i32 + dx).max(0) as u32;
            let y = (station_pos.y as i32 + dy).max(0) as u32;

            if x < map.width && y < map.height {
                if let Some(tile) = map.get_tile_mut(x, y) {
                    *tile = Tile::Empty; 
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionCosts {
    pub explorer: (u32, u32),
    pub energy_collector: (u32, u32),
    pub miner: (u32, u32),
    pub scientist: (u32, u32),
}

// / Implémentation de la station
impl Station {
    pub fn new(global_map: &mut Map) -> Self {
        let preferred_pos = Position { x: 10, y: 10 };

        let pos = find_nearby_empty_position(global_map, preferred_pos);

        clear_area_around_station(global_map, &pos);

        if let Some(tile) = global_map.get_tile_mut(pos.x, pos.y) {
            *tile = Tile::Station;
        }

        Self {
            position: pos,
            resources: Resources {
                energy: 10000,
                minerals: 5000,
                scientific_data: 0,
            },
            known_map: None,
            robots: Vec::new(),
            max_robots: 10,
            production_costs: ProductionCosts {
                explorer: (20, 100),
                energy_collector: (150, 150),
                miner: (150, 200),
                scientist: (250, 150),
            },
            discovered_resources: DiscoveredResources::default(),
        }
    }

    pub fn update(&mut self, _global_map: &Map) {
        self.sync_with_returned_robots();
    }

    fn sync_with_returned_robots(&mut self) {
    }

    pub fn collect_robot_resources(&mut self, robot: &mut Robot) {
        self.resources.energy += robot.inventory.energy;
        self.resources.minerals += robot.inventory.minerals;
        self.resources.scientific_data += robot.inventory.scientific_data;

        robot.inventory.energy = 0;
        robot.inventory.minerals = 0;
        robot.inventory.scientific_data = 0;
    }

    pub fn can_create_robot(&self, robot_type: RobotType) -> bool {
        match robot_type {
            RobotType::Explorer => {
                let (energy_cost, mineral_cost) = self.production_costs.explorer;
                self.resources.energy >= energy_cost && self.resources.minerals >= mineral_cost
            }
            RobotType::EnergyCollector => {
                let (energy_cost, mineral_cost) = self.production_costs.energy_collector;
                self.resources.energy >= energy_cost && self.resources.minerals >= mineral_cost
            }
            RobotType::Miner => {
                let (energy_cost, mineral_cost) = self.production_costs.miner;
                self.resources.energy >= energy_cost && self.resources.minerals >= mineral_cost
            }
            RobotType::Scientist => {
                let (energy_cost, mineral_cost) = self.production_costs.scientist;
                self.resources.energy >= energy_cost && self.resources.minerals >= mineral_cost
            }
        }
    }

    /// Crée un nouveau robot si les ressources sont suffisantes
    pub fn create_robot(&mut self, robot_type: RobotType) -> Option<Robot> {
        if self.robots.len() >= self.max_robots {
            return None;
        }

        if !self.can_create_robot(robot_type) {
            return None;
        }

        let modules = self.get_modules_for_robot_type(robot_type);

        self.consume_resources_for_robot(robot_type);

        let energy_cost = match robot_type {
            RobotType::Explorer => self.production_costs.explorer.0,
            RobotType::EnergyCollector => self.production_costs.energy_collector.0,
            RobotType::Miner => self.production_costs.miner.0,
            RobotType::Scientist => self.production_costs.scientist.0,
        };

        let mut robot = Robot::new(
            Position {
                x: self.position.x.saturating_add(1),
                y: self.position.y,
            },
            modules,
        );

        robot.energy = energy_cost as f32;

        self.robots.push(robot.clone());
        Some(robot)
    }


    /// Retourne les modules appropriés pour un type de robot
    fn get_modules_for_robot_type(&self, robot_type: RobotType) -> Vec<HardwareModule> {
        match robot_type {
            RobotType::Explorer => vec![HardwareModule::TerrainScanner {
                efficiency: 0.9,
                range: 20,
            }],
            RobotType::EnergyCollector => vec![
                HardwareModule::EnergyHarvester {
                    collection_rate: 2.0,
                },
                HardwareModule::TerrainScanner {
                    efficiency: 0.6,
                    range: 10,
                },
            ],
            RobotType::Miner => vec![
                HardwareModule::DeepDrill { mining_speed: 2.0 },
                HardwareModule::TerrainScanner {
                    efficiency: 0.6,
                    range: 10,
                },
            ],
            RobotType::Scientist => vec![
                HardwareModule::SpectralAnalyzer {
                    analysis_accuracy: 0.95,
                },
                HardwareModule::TerrainScanner {
                    efficiency: 0.7,
                    range: 15,
                },
            ],
        }
    }

    /// Consomme les ressources nécessaires pour créer un robot
    fn consume_resources_for_robot(&mut self, robot_type: RobotType) {
        match robot_type {
            RobotType::Explorer => {
                let (energy_cost, mineral_cost) = self.production_costs.explorer;
                self.resources.energy -= energy_cost;
                self.resources.minerals -= mineral_cost;
            }
            RobotType::EnergyCollector => {
                let (energy_cost, mineral_cost) = self.production_costs.energy_collector;
                self.resources.energy -= energy_cost;
                self.resources.minerals -= mineral_cost;
            }
            RobotType::Miner => {
                let (energy_cost, mineral_cost) = self.production_costs.miner;
                self.resources.energy -= energy_cost;
                self.resources.minerals -= mineral_cost;
            }
            RobotType::Scientist => {
                let (energy_cost, mineral_cost) = self.production_costs.scientist;
                self.resources.energy -= energy_cost;
                self.resources.minerals -= mineral_cost;
            }
        }
    }

    // Méthode pour signaler une ressource découverte
    pub fn report_resource_found(&mut self, resource_type: ResourceType, position: Position) {
        match resource_type {
            ResourceType::Energy => {
                if !self
                    .discovered_resources
                    .energy_locations
                    .contains(&position)
                {
                    self.discovered_resources.energy_locations.push(position);
                }
            }
            ResourceType::Minerals => {
                if !self
                    .discovered_resources
                    .mineral_locations
                    .contains(&position)
                {
                    self.discovered_resources.mineral_locations.push(position);
                }
            }
            ResourceType::ScientificData => {
                if !self
                    .discovered_resources
                    .scientific_locations
                    .contains(&position)
                {
                    self.discovered_resources
                        .scientific_locations
                        .push(position);
                }
            }
        }
    }

    // Récupère le nombre de ressources découvertes
    pub fn get_discovered_resource_counts(&self) -> (usize, usize, usize) {
        (
            self.discovered_resources.energy_locations.len(),
            self.discovered_resources.mineral_locations.len(),
            self.discovered_resources.scientific_locations.len(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RobotType {
    Explorer,        
    EnergyCollector, 
    Miner,           
    Scientist,       
}
