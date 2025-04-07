use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HardwareModule {
    TerrainScanner {
        efficiency: f32,
        range: u32,
    },
    DeepDrill {
        mining_speed: f32,
    },
    SpectralAnalyzer {
        analysis_accuracy: f32,
    },
    EnergyHarvester {
        collection_rate: f32,
    },
}