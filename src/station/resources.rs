#[allow(dead_code)]
use crate::robot::Resources;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesHistory {
    pub energy_collected: Vec<(u64, u32)>,
    pub minerals_collected: Vec<(u64, u32)>,
    pub scientific_data_collected: Vec<(u64, u32)>,
    pub resources_used_for_production: Vec<(u64, Resources)>,
}

impl ResourcesHistory {
    pub fn new() -> Self {
        Self {
            energy_collected: Vec::new(),
            minerals_collected: Vec::new(),
            scientific_data_collected: Vec::new(),
            resources_used_for_production: Vec::new(),
        }
    }

    pub fn add_energy_collected(&mut self, timestamp: u64, amount: u32) {
        self.energy_collected.push((timestamp, amount));
    }

    pub fn add_minerals_collected(&mut self, timestamp: u64, amount: u32) {
        self.minerals_collected.push((timestamp, amount));
    }

    pub fn add_scientific_data_collected(&mut self, timestamp: u64, amount: u32) {
        self.scientific_data_collected.push((timestamp, amount));
    }

    pub fn add_resources_used_for_production(&mut self, timestamp: u64, resources: Resources) {
        self.resources_used_for_production
            .push((timestamp, resources));
    }

    pub fn total_collected(&self) -> Resources {
        let total_energy = self.energy_collected.iter().map(|(_, amount)| amount).sum();
        let total_minerals = self
            .minerals_collected
            .iter()
            .map(|(_, amount)| amount)
            .sum();
        let total_scientific_data = self
            .scientific_data_collected
            .iter()
            .map(|(_, amount)| amount)
            .sum();

        Resources {
            energy: total_energy,
            minerals: total_minerals,
            scientific_data: total_scientific_data,
        }
    }

    pub fn total_used_for_production(&self) -> Resources {
        let mut total = Resources {
            energy: 0,
            minerals: 0,
            scientific_data: 0,
        };

        for (_, resources) in &self.resources_used_for_production {
            total.energy += resources.energy;
            total.minerals += resources.minerals;
            total.scientific_data += resources.scientific_data;
        }

        total
    }
}
