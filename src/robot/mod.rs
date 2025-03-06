pub mod module; // Hardware modules
pub mod position; // Position handling
pub mod resources;
pub mod state; // State machine // Resource management

pub use module::HardwareModule;
pub use position::Position;
pub use resources::Resources;
pub use state::State;

use crate::map::Map;
use rand::Rng;
use serde::{Deserialize, Serialize};

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
    pub fn move_randomly(&mut self, map: &Map) {
        let mut rng = rand::thread_rng();

        // Generate random direction
        let dx = rng.gen_range(-1..=1);
        let dy = rng.gen_range(-1..=1);

        let new_x = self.position.x.saturating_add_signed(dx);
        let new_y = self.position.y.saturating_add_signed(dy);

        // Check if the new position is valid
        if new_x < map.width && new_y < map.height && !map.is_obstacle(new_x, new_y) {
            self.position.x = new_x;
            self.position.y = new_y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::{Map, Tile};

    #[test]
    fn test_robot_initialization() {
        let pos = Position { x: 5, y: 5 };
        let modules = vec![HardwareModule::TerrainScanner {
            efficiency: 0.8,
            range: 10,
        }];
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
    fn test_move_randomly_obstacle_avoidance() {
        // Create a 3x3 map with center obstacle
        let mut map = Map::new(3, 3, 12345);

        // Create obstacle ring around center using existing methods
        for y in 0..3 {
            for x in 0..3 {
                if x == 1 && y == 1 {
                    continue; // Keep center empty
                }
                if let Some(tile) = map.get_tile_mut(x, y) {
                    *tile = Tile::Obstacle;
                }
            }
        }

        let start_pos = Position { x: 1, y: 1 };
        let mut robot = Robot::new(start_pos, vec![]);

        // Try moving 10 times - should stay in center
        for _ in 0..10 {
            robot.move_randomly(&map);
            assert_eq!(
                robot.position, start_pos,
                "Robot should not move from surrounded position"
            );
        }
    }
}
