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

use crate::{map::{Map, Tile}, station::Station};
use rand::{seq::SliceRandom, Rng};
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
    /// List of visited positions by the robot.
    pub visited_positions: Vec<Position>,
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
            visited_positions: vec![initial_pos], // Initialiser avec la position de départ
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
        // Explore la carte en recherchant des ressources
        pub fn explore_map(&mut self, map: &Map, station: &mut Station) {
            // Ne retourner à la station que si l'énergie est vraiment basse
            if self.energy < 10.0 {  // Changé de 20.0 à 10.0 pour explorer plus longtemps
                self.state = State::Returning {
                    base_position: station.position,
                };
                return;
            }
    
            // Obtenir les coordonnées actuelles
            let current_x = self.position.x;
            let current_y = self.position.y;
            
            // Explorer la tuile actuelle et la signaler à la station
            if let Some(tile) = map.get_tile(current_x, current_y) {
                // Vérifier si la tuile contient des ressources
                match tile {
                    Tile::Energy(_) => {
                        // Signaler à la station qu'on a trouvé de l'énergie
                        station.report_resource_found(ResourceType::Energy, self.position);
                    },
                    Tile::Mineral(_) => {
                        // Signaler à la station qu'on a trouvé un minéral
                        station.report_resource_found(ResourceType::Minerals, self.position);
                    },
                    Tile::ScientificPoint(_) => {
                        // Signaler à la station qu'on a trouvé un point scientifique
                        station.report_resource_found(ResourceType::ScientificData, self.position);
                    },
                    _ => {}
                }
            }

            // Ajouter cette ligne après le déplacement stratégique
            self.strategic_move(map);
            
            // Enregistrer la position actuelle comme visitée
            if !self.visited_positions.contains(&self.position) {
                self.visited_positions.push(self.position);
            }
            
            // Consommer de l'énergie pour le déplacement
            self.energy -= 1.0;
        }
    
        // Déplacement stratégique pour l'exploration
        fn strategic_move(&mut self, map: &Map) {
            let mut rng = rand::thread_rng();
            
            // Direction préférée: s'éloigner davantage pour une meilleure exploration
            let directions = [
                (0, 1),  // droite
                (1, 0),  // bas
                (0, -1), // gauche
                (-1, 0), // haut
                (1, 1),  // diagonale bas-droite
                (1, -1), // diagonale bas-gauche
                (-1, 1), // diagonale haut-droite
                (-1, -1), // diagonale haut-gauche
            ];
            
            // Augmenter la distance de déplacement (4-8 cases au lieu de 2-3)
            let distance = rng.gen_range(4..=8);
            
            // Mélanger les directions pour avoir du mouvement aléatoire
            let mut shuffled_directions = directions.to_vec();
            shuffled_directions.shuffle(&mut rng);
            
            // Explorer dans une direction jusqu'à ce qu'on trouve une position valide
            for (dx, dy) in shuffled_directions {
                // Calculer plusieurs positions intermédiaires pour éviter de sauter par-dessus des obstacles
                let step_size = 2;
                let mut final_x = self.position.x;
                let mut final_y = self.position.y;
                
                // Avancer par pas de 2 jusqu'à la distance souhaitée ou jusqu'à rencontrer un obstacle
                for _step in 1..=(distance/step_size) {
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
                        // Position non valide, arrêter l'avancée
                        break;
                    }
                }
                
                // Si nous avons pu nous déplacer d'au moins une case, utiliser cette position
                if final_x != self.position.x || final_y != self.position.y {
                    self.position.x = final_x;
                    self.position.y = final_y;
                    break;
                }
            }
        }
        
        // Retour à la station
        pub fn return_to_station(&mut self, station: &Station) {
            // Calculer le vecteur vers la station
            let dx = station.position.x as i32 - self.position.x as i32;
            let dy = station.position.y as i32 - self.position.y as i32;
            
            // Normaliser le vecteur pour obtenir une direction
            let dir_x = dx.signum();
            let dir_y = dy.signum();
            
            // Déplacer le robot d'un pas vers la station
            self.position.x = self.position.x.saturating_add_signed(dir_x);
            self.position.y = self.position.y.saturating_add_signed(dir_y);
            
            // Consommer de l'énergie pour le déplacement
            self.energy -= 0.5;
        }
        
        // Vérifie si le robot est à la station
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
