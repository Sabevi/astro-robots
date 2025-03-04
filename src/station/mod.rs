pub mod production;
pub mod resources;
pub mod sync;

use crate::map::Map;
use crate::robot::{HardwareModule, Position, Resources, Robot};
use serde::{Deserialize, Serialize};

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
    pub fn new(position: Position) -> Self {
        Self {
            position,
            resources: Resources {
                energy: 1000, // Ressources initiales
                minerals: 500,
                scientific_data: 0,
            },
            known_map: None,
            robots: Vec::new(),
            max_robots: 10, // Limite initiale
            production_costs: ProductionCosts {
                explorer: (200, 100),
                energy_collector: (150, 150),
                miner: (150, 200),
                scientist: (250, 150),
            },
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
}

/// Enum représentant les différents types de robots que la station peut créer
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RobotType {
    Explorer,        // Robot pour explorer et cartographier
    EnergyCollector, // Robot pour collecter de l'énergie
    Miner,           // Robot pour collecter des minéraux
    Scientist,       // Robot pour collecter des données scientifiques
}
