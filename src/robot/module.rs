use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HardwareModule {
    // a hardware to scan the terrain
    TerrainScanner {
        efficiency: f32,
        range: u32,
    },
    // a hardware to extract resources
    DeepDrill {
        mining_speed: f32,
    },
    // a hardware to analyze scientific samples
    SpectralAnalyzer {
        analysis_accuracy: f32,
    },
    // a hardware to collect energy
    EnergyHarvester {
        collection_rate: f32,
    },
}