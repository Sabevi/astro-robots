#[allow(dead_code)]
use crate::robot::Resources;
use serde::{Deserialize, Serialize};

/// Structure pour suivre la production et la consommation de ressources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesHistory {
    /// Historique de l'énergie collectée au fil du temps
    pub energy_collected: Vec<(u64, u32)>, // (timestamp, amount)
    /// Historique des minéraux collectés au fil du temps
    pub minerals_collected: Vec<(u64, u32)>,
    /// Historique des données scientifiques collectées au fil du temps
    pub scientific_data_collected: Vec<(u64, u32)>,
    /// Historique des ressources utilisées pour la production de robots
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

    /// Ajoute un événement de collecte d'énergie
    pub fn add_energy_collected(&mut self, timestamp: u64, amount: u32) {
        self.energy_collected.push((timestamp, amount));
    }

    /// Ajoute un événement de collecte de minéraux
    pub fn add_minerals_collected(&mut self, timestamp: u64, amount: u32) {
        self.minerals_collected.push((timestamp, amount));
    }

    /// Ajoute un événement de collecte de données scientifiques
    pub fn add_scientific_data_collected(&mut self, timestamp: u64, amount: u32) {
        self.scientific_data_collected.push((timestamp, amount));
    }

    /// Ajoute un événement d'utilisation de ressources pour la production
    pub fn add_resources_used_for_production(&mut self, timestamp: u64, resources: Resources) {
        self.resources_used_for_production
            .push((timestamp, resources));
    }

    /// Calcule le total des ressources collectées
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

    /// Calcule le total des ressources utilisées pour la production
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
