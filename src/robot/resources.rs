use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    Energy,
    Minerals,
    ScientificData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resources {
    pub energy: u32,
    pub minerals: u32,
    pub scientific_data: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(PartialEq)]
pub struct ScientificSample {
    pub data_type: String,
    pub value: f64,
    pub coordinates: (u32, u32),
}