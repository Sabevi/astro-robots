#[allow(unused_imports)]
use crate::robot::{HardwareModule, Position, Robot};
use crate::station::RobotType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionQueue {
    /// File d'attente des robots à produire
    queue: Vec<RobotType>,
    /// Temps de production pour chaque type de robot (en ticks)
    production_time: ProductionTime,
    /// Temps restant pour la production du robot actuel
    current_production_time_left: Option<u32>,
    /// Type de robot en cours de production
    current_production: Option<RobotType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionTime {
    /// Temps pour produire un explorateur (en ticks)
    pub explorer: u32,
    /// Temps pour produire un collecteur d'énergie (en ticks)
    pub energy_collector: u32,
    /// Temps pour produire un mineur (en ticks)
    pub miner: u32,
    /// Temps pour produire un scientifique (en ticks)
    pub scientist: u32,
}

impl ProductionQueue {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
            production_time: ProductionTime {
                explorer: 50,
                energy_collector: 40,
                miner: 45,
                scientist: 60,
            },
            current_production_time_left: None,
            current_production: None,
        }
    }

    /// Ajoute un robot à la file d'attente de production
    pub fn enqueue(&mut self, robot_type: RobotType) {
        self.queue.push(robot_type);
    }

    /// Met à jour la production de robots et retourne un robot terminé s'il y en a un
    pub fn update(&mut self) -> Option<RobotType> {
        if self.current_production.is_none() && !self.queue.is_empty() {
            // Commencer une nouvelle production
            let robot_type = self.queue.remove(0);
            self.current_production = Some(robot_type);
            self.current_production_time_left = Some(self.get_production_time(robot_type));
        }

        if let Some(time_left) = self.current_production_time_left.as_mut() {
            if *time_left > 0 {
                *time_left -= 1;
            } else {
                // Production terminée
                let completed_robot = self.current_production.take();
                self.current_production_time_left = None;
                return completed_robot;
            }
        }

        None
    }

    /// Retourne le temps nécessaire pour produire un type de robot spécifique
    fn get_production_time(&self, robot_type: RobotType) -> u32 {
        match robot_type {
            RobotType::Explorer => self.production_time.explorer,
            RobotType::EnergyCollector => self.production_time.energy_collector,
            RobotType::Miner => self.production_time.miner,
            RobotType::Scientist => self.production_time.scientist,
        }
    }

    /// Retourne le nombre de robots dans la file d'attente
    pub fn queue_size(&self) -> usize {
        self.queue.len()
    }

    /// Retourne le pourcentage de progression de la production actuelle
    pub fn production_progress(&self) -> Option<f32> {
        if let (Some(robot_type), Some(time_left)) =
            (self.current_production, self.current_production_time_left)
        {
            let total_time = self.get_production_time(robot_type) as f32;
            let progress = (total_time - time_left as f32) / total_time;
            Some(progress)
        } else {
            None
        }
    }
}
