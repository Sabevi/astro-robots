#[allow(dead_code)]
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
    /// Position sur la carte
    pub position: Position,
    /// Ressources stockées dans la station
    pub resources: Resources,
    /// Map connue par la station (pour simuler la synchronisation git-like)
    pub known_map: Option<Map>,
    /// Robots créés par la station
    pub robots: Vec<Robot>,
    /// Capacité maximale de robots
    pub max_robots: usize,
    /// Coûts de production pour chaque type de robot
    pub production_costs: ProductionCosts,
    /// Ressources découvertes
    pub discovered_resources: DiscoveredResources,
}

fn find_nearby_empty_position(map: &Map, center: Position) -> Position {
    let max_radius = map.width.max(map.height) as i32; // pour éviter les boucles infinies
    for radius in 0..max_radius {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                // Bord de la couronne uniquement
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

    // Fallback si tout est bloqué
    Position { x: 0, y: 0 }
}

fn clear_area_around_station(map: &mut Map, station_pos: &Position) {
    let radius = 3; // Rayon de nettoyage pour éviter que la station soit bloquée

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let x = (station_pos.x as i32 + dx).max(0) as u32;
            let y = (station_pos.y as i32 + dy).max(0) as u32;

            if x < map.width && y < map.height {
                if let Some(tile) = map.get_tile_mut(x, y) {
                    *tile = Tile::Empty; // Remplace tout par du vide
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionCosts {
    /// Coût en énergie et minéraux pour un robot explorateur
    pub explorer: (u32, u32),
    /// Coût en énergie et minéraux pour un robot collecteur d'énergie
    pub energy_collector: (u32, u32),
    /// Coût en énergie et minéraux pour un robot mineur
    pub miner: (u32, u32),
    /// Coût en énergie et minéraux pour un robot scientifique
    pub scientist: (u32, u32),
}

impl Station {
    /// Crée une nouvelle station à la position donnée
    pub fn new(global_map: &mut Map) -> Self {
        let preferred_pos = Position { x: 10, y: 10 };

        let pos = find_nearby_empty_position(global_map, preferred_pos);

        // Nettoyer la zone autour AVANT de placer la station
        clear_area_around_station(global_map, &pos);

        // Placer la station après nettoyage
        if let Some(tile) = global_map.get_tile_mut(pos.x, pos.y) {
            *tile = Tile::Station;
        }

        // Retourner l'instance de la station
        Self {
            position: pos,
            resources: Resources {
                energy: 1000,
                minerals: 500,
                scientific_data: 0,
            },
            known_map: None,
            robots: Vec::new(),
            max_robots: 10,
            production_costs: ProductionCosts {
                explorer: (200, 100),
                energy_collector: (150, 150),
                miner: (150, 200),
                scientist: (250, 150),
            },
            discovered_resources: DiscoveredResources::default(),
        }

    }

    /// Met à jour l'état de la station
    pub fn update(&mut self, _global_map: &Map) {
        // Mettre à jour la logique de la station ici
        self.sync_with_returned_robots();
    }

    /// Synchronise les données avec les robots qui sont revenus à la station
    fn sync_with_returned_robots(&mut self) {
        // À implémenter: logique de synchronisation "git-like"
    }

    /// Collecte les ressources des robots revenus à la station
    pub fn collect_robot_resources(&mut self, robot: &mut Robot) {
        // Ajouter les ressources du robot à la station
        self.resources.energy += robot.inventory.energy;
        self.resources.minerals += robot.inventory.minerals;
        self.resources.scientific_data += robot.inventory.scientific_data;

        // Vider l'inventaire du robot
        robot.inventory.energy = 0;
        robot.inventory.minerals = 0;
        robot.inventory.scientific_data = 0;
    }

    /// Vérifie si un robot peut être créé selon les ressources disponibles
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
            return None; // Capacité maximale atteinte
        }

        if !self.can_create_robot(robot_type) {
            return None; // Ressources insuffisantes
        }

        // Créer les modules en fonction du type de robot
        let modules = self.get_modules_for_robot_type(robot_type);

        // Consommer les ressources
        self.consume_resources_for_robot(robot_type);

        // Créer le robot avec une position proche de la station
        let robot = Robot::new(
            Position {
                x: self.position.x.saturating_add(1),
                y: self.position.y,
            },
            modules,
        );

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
                if !self.discovered_resources.energy_locations.contains(&position) {
                    self.discovered_resources.energy_locations.push(position);
                }
            },
            ResourceType::Minerals => {
                if !self.discovered_resources.mineral_locations.contains(&position) {
                    self.discovered_resources.mineral_locations.push(position);
                }
            },
            ResourceType::ScientificData => {
                if !self.discovered_resources.scientific_locations.contains(&position) {
                    self.discovered_resources.scientific_locations.push(position);
                }
            },
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

/// Enum représentant les différents types de robots que la station peut créer
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RobotType {
    Explorer,        // Robot pour explorer et cartographier
    EnergyCollector, // Robot pour collecter de l'énergie
    Miner,           // Robot pour collecter des minéraux
    Scientist,       // Robot pour collecter des données scientifiques
}
