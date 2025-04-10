pub mod communication;
pub mod module;
pub mod position;
pub mod resources;
pub mod state;

pub use module::HardwareModule;
pub use position::Position;
use resources::ResourceType;
pub use resources::Resources;
pub use state::State;

use crate::{
    map::{Map, Tile},
    station::Station,
};
use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Robot {
    pub position: Position,
    pub state: State,
    pub energy: f32,
    pub modules: Vec<HardwareModule>,
    pub inventory: Resources,
    pub visited_positions: Vec<Position>,
    pub steps_since_last_energy: u32,
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
            visited_positions: vec![initial_pos], 
            steps_since_last_energy: 0,
        }
    }
    pub fn move_randomly(&mut self, map: &Map) {
        let mut rng = rand::thread_rng();

        // Generate random direction
        let dx = rng.gen_range(-1..=1);
        let dy = rng.gen_range(-1..=1);

        let new_x = self.position.x.saturating_add_signed(dx);
        let new_y = self.position.y.saturating_add_signed(dy);

        if new_x < map.width && new_y < map.height && !map.is_obstacle(new_x, new_y) {
            self.position.x = new_x;
            self.position.y = new_y;
        }
    }
    // Explore la carte en recherchant des ressources
    pub fn explore_map(&mut self, map: &Map, station: &mut Station) {
        if self.energy < 10.0 {
            self.state = State::Returning {
                base_position: station.position,
            };
            return;
        }

        self.steps_since_last_energy += 1;
        if self.steps_since_last_energy >= 15 {
            self.energy -= 10.0;
            self.steps_since_last_energy = 0;
        }

        let distance_to_base = self.position.distance_to(&station.position);
        let energy_needed_to_return = distance_to_base as f32 * 0.5;

        if self.energy <= energy_needed_to_return + 1.0 {
            self.state = State::Returning {
                base_position: station.position,
            };
            return;
        }

        let current_x = self.position.x;
        let current_y = self.position.y;

        if let Some(tile) = map.get_tile(current_x, current_y) {
            match tile {
                Tile::Energy(_) => {
                    station.report_resource_found(ResourceType::Energy, self.position);
                }
                Tile::Mineral(_) => {
                    station.report_resource_found(ResourceType::Minerals, self.position);
                }
                Tile::ScientificPoint(_) => {
                    station.report_resource_found(ResourceType::ScientificData, self.position);
                }
                _ => {}
            }
        }

        self.strategic_move(map);

        if !self.visited_positions.contains(&self.position) {
            self.visited_positions.push(self.position);
        }
    }

    fn strategic_move(&mut self, map: &Map) {
        let mut rng = rand::thread_rng();

        let directions = [
            (0, 1),   
            (1, 0),   
            (0, -1),  
            (-1, 0),  
            (1, 1),   
            (1, -1),  
            (-1, 1),  
            (-1, -1), 
        ];

        let distance = rng.gen_range(4..=8);

        let mut shuffled_directions = directions.to_vec();
        shuffled_directions.shuffle(&mut rng);

        // Explorer dans une direction jusqu'à ce qu'on trouve une position valide
        for (dx, dy) in shuffled_directions {
            let step_size = 2;
            let mut final_x = self.position.x;
            let mut final_y = self.position.y;

            for _step in 1..=(distance / step_size) {
                let new_x = match dx {
                    v if v > 0 => final_x.saturating_add(step_size),
                    v if v < 0 => final_x.saturating_sub(step_size),
                    _ => final_x,
                };

                let new_y = match dy {
                    v if v > 0 => final_y.saturating_add(step_size),
                    v if v < 0 => final_y.saturating_sub(step_size),
                    _ => final_y,
                };

                // Vérifier si la position est valide
                if new_x < map.width && new_y < map.height && !map.is_obstacle(new_x, new_y) {
                    final_x = new_x;
                    final_y = new_y;
                } else {
                    break;
                }
            }

            if final_x != self.position.x || final_y != self.position.y {
                self.position.x = final_x;
                self.position.y = final_y;
                break;
            }
        }
    }

    // Retour à la station
    pub fn return_to_station(&mut self, station: &Station) {
        let dx = station.position.x as i32 - self.position.x as i32;
        let dy = station.position.y as i32 - self.position.y as i32;

        let dir_x = dx.signum();
        let dir_y = dy.signum();

        self.position.x = self.position.x.saturating_add_signed(dir_x);
        self.position.y = self.position.y.saturating_add_signed(dir_y);

        self.energy -= 0.5;
    }

    pub fn is_at_station(&self, station: &Station) -> bool {
        self.position.x == station.position.x && self.position.y == station.position.y
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
        let mut map = Map::new(3, 3, 12345);

        for y in 0..3 {
            for x in 0..3 {
                if x == 1 && y == 1 {
                    continue; 
                }
                if let Some(tile) = map.get_tile_mut(x, y) {
                    *tile = Tile::Obstacle;
                }
            }
        }

        let start_pos = Position { x: 1, y: 1 };
        let mut robot = Robot::new(start_pos, vec![]);

        for _ in 0..10 {
            robot.move_randomly(&map);
            assert_eq!(
                robot.position, start_pos,
                "Robot should not move from surrounded position"
            );
        }
    }
}
