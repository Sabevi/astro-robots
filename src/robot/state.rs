use serde::{Serialize, Deserialize};

use crate::robot::position::Position;
use crate::robot::resources::ScientificSample;
use crate::robot::resources::ResourceType;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum State {
    Idle,
    Exploring {
        target: Position,
        path: Vec<Position>,
    },
    Collecting {
        resource_type: ResourceType,
        target: Position,
    },
    Analyzing {
        sample: ScientificSample,
    },
    Returning {
        base_position: Position,
    },
}