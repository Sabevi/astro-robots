pub mod position;    // Position handling
pub mod state;       // State machine
pub mod module;      // Hardware modules
pub mod resources;   // Resource management

pub use position::Position;
pub use state::State;
pub use module::HardwareModule;
pub use resources::Resources;

use crate::map::Map;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Robot {
    pub position: Position,
    /// Current operational state of the robot (e.g., Exploring).
    pub state: State,
    /// Remaining energy level of the robot.
    pub energy: f32,
    /// List of hardware modules equipped on this robot.
    pub modules: Vec<HardwareModule>,
    /// Inventory of resources collected by this robot.
    pub inventory: Resources,
}

impl Robot {
    pub fn new(initial_pos: Position, modules: Vec<HardwareModule>) -> Self {
        Self {
            position: initial_pos,
            state: State::Idle,
            energy: 100.0,
            modules,
            inventory: Resources {
                energy: 0,
                minerals: 0,
                scientific_data: 0,
            },
        }
    }

    pub fn update(&mut self, map: &Map) {
        match &self.state {
            State::Exploring { target , path: _ } => {
                let target = target.clone();
                self.move_towards(&target, map)
            },
            _ => {}
        }
    }

    fn move_towards(&mut self, target: &Position, map: &Map) {
        let dx = target.x as f32 - self.position.x as f32;
        let dy = target.y as f32 - self.position.y as f32;
        let distance = (dx.hypot(dy)).max(1.0);

        let step_x = (dx / distance).round() as i32;
        let step_y = (dy / distance).round() as i32;

        let new_x = self.position.x.saturating_add_signed(step_x);
        let new_y = self.position.y.saturating_add_signed(step_y);

        if !map.is_obstacle(new_x, new_y) {
            self.position.x = new_x;
            self.position.y = new_y;
        }
    }

    pub fn start_exploring(&mut self, target: Position) {
        self.state = State::Exploring { target, path: vec![] };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a simple map for testing
    fn create_test_map() -> Map {
        Map::new(10, 10, 12345) // Assuming Map::new(width, height, seed) exists
    }

    #[test]
    fn test_robot_initialization() {
        let pos = Position { x: 5, y: 5 };
        let modules = vec![HardwareModule::TerrainScanner { efficiency: 0.8, range: 10 }];
        let robot = Robot::new(pos, modules.clone());
        
        assert_eq!(robot.position, pos);
        assert!(matches!(robot.state, State::Idle));
        assert_eq!(robot.energy, 100.0);
        assert_eq!(robot.modules, modules);
        assert_eq!(robot.inventory.energy, 0);
        assert_eq!(robot.inventory.minerals, 0);
        assert_eq!(robot.inventory.scientific_data, 0);
    }

    #[test]
    fn test_start_exploring() {
        let mut robot = Robot::new(Position { x: 0, y: 0 }, vec![]);
        let target = Position { x: 5, y: 5 };
        robot.start_exploring(target);
        
        match robot.state {
            State::Exploring { target: explore_target, path } => {
                assert_eq!(explore_target, target);
                assert!(path.is_empty());
            },
            _ => panic!("Robot should be in Exploring state"),
        }
    }

    #[test]
    fn test_move_towards() {
        let mut robot = Robot::new(Position { x: 0, y: 0 }, vec![]);
        let target = Position { x: 3, y: 4 };
        let map = create_test_map();
        
        robot.move_towards(&target, &map);
        
        // The robot should move diagonally towards the target
        assert_eq!(robot.position, Position { x: 1, y: 1 });
    }

    #[test]
    fn test_update_exploring() {
        let mut robot = Robot::new(Position { x: 0, y: 0 }, vec![]);
        let target = Position { x: 3, y: 4 };
        robot.start_exploring(target);
        let map = create_test_map();
        
        robot.update(&map);
        
        // The robot should have moved towards the target
        assert_ne!(robot.position, Position { x: 0, y: 0 });
    }
    // add tests for obstacle avoidance
}
