#[allow(unused_imports)]
use crate::robot::{HardwareModule, Position, Robot};
use crate::station::RobotType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionQueue {
    queue: Vec<RobotType>,
    production_time: ProductionTime,
    current_production_time_left: Option<u32>,
    current_production: Option<RobotType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionTime {
    pub explorer: u32,
    pub energy_collector: u32,
    pub miner: u32,
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

    pub fn enqueue(&mut self, robot_type: RobotType) {
        self.queue.push(robot_type);
    }

    /// Met à jour la production de robots et retourne un robot terminé s'il y en a un
    pub fn update(&mut self) -> Option<RobotType> {
        if self.current_production.is_none() && !self.queue.is_empty() {
            let robot_type = self.queue.remove(0);
            self.current_production = Some(robot_type);
            self.current_production_time_left = Some(self.get_production_time(robot_type));
        }

        if let Some(time_left) = self.current_production_time_left.as_mut() {
            if *time_left > 0 {
                *time_left -= 1;
            } else {
                let completed_robot = self.current_production.take();
                self.current_production_time_left = None;
                return completed_robot;
            }
        }

        None
    }

    fn get_production_time(&self, robot_type: RobotType) -> u32 {
        match robot_type {
            RobotType::Explorer => self.production_time.explorer,
            RobotType::EnergyCollector => self.production_time.energy_collector,
            RobotType::Miner => self.production_time.miner,
            RobotType::Scientist => self.production_time.scientist,
        }
    }

    pub fn queue_size(&self) -> usize {
        self.queue.len()
    }

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
