// ========================= IMPORTS ========================= //
use noise::{NoiseFn, Value};
use serde::{Serialize, Deserialize};
pub mod map_widget;  

// ========================= CONSTANTS ========================= //
const OBSTACLE_THRESHOLD: f64 = 0.4; 
const NOISE_SCALE: f64 = 0.2;

// ========================= MAP STRUCTURE ========================= //
// Represents the entire game map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub width: u32,
    pub height: u32,
    tiles: Vec<Tile>,
    pub seed: u64,
}

// ========================= TILE ENUM ========================= //
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Tile {
    Empty,    // Walkable space with no obstacle
    Obstacle,
}


// ========================= MAP IMPLEMENTATION ========================= //
impl Map {
    /// Creates a new map with the given dimensions and seed.
    pub fn new(width: u32, height: u32, seed: u64) -> Self {
        let mut tiles = Vec::with_capacity((width * height) as usize);
        let noise = Value::new(seed as u32);

        // Generate the map tiles
        for y in 0..height {
            for x in 0..width {
                let noise_value = noise.get([x as f64 * NOISE_SCALE, y as f64 * NOISE_SCALE]);

                // Determine if the tile is an obstacle or an empty space
                let tile = if noise_value > OBSTACLE_THRESHOLD {
                    Tile::Obstacle
                } else {
                    Tile::Empty
                };
                
                tiles.push(tile);
            }
        }

        
        // Return the fully generated map
        Map {
            width,
            height,
            tiles,
            seed,
        }
    }

    /// Retrieves a tile at the given coordinates (read-only).
    pub fn get_tile(&self, x: u32, y: u32) -> Option<&Tile> {
        if x >= self.width || y >= self.height {
            return None; // Return None if out of bounds
        }
        self.tiles.get((y * self.width + x) as usize)
    }

    /// Retrieves a mutable reference to a tile at the given coordinates.
    pub fn get_tile_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        if x >= self.width || y >= self.height {
            return None; // Return None if out of bounds
        }
        self.tiles.get_mut((y * self.width + x) as usize)
    }

    /// Checks whether the given coordinates contain an obstacle.
    pub fn is_obstacle(&self, x: u32, y: u32) -> bool {
        matches!(self.get_tile(x, y), Some(Tile::Obstacle))
    }
}

// ===================================================================
// ====================== UNIT TESTS ============================
// ===================================================================
#[cfg(test)]
mod tests {
    use super::*;

    /// Test that a map is correctly generated with the given dimensions.
    #[test]
    fn test_map_generation() {
        let width = 10;
        let height = 10;
        let seed = 42;
        let map = Map::new(width, height, seed);

        // Check that map dimensions are correct
        assert_eq!(map.width, width, "Map width should be correct.");
        assert_eq!(map.height, height, "Map height should be correct.");
        assert_eq!(map.tiles.len(), (width * height) as usize, "Total number of tiles should match dimensions.");
    }

    /// Test that at least some obstacles are generated.
    #[test]
    fn test_obstacles_generation() {
        let width = 10;
        let height = 10;
        let seed = 42;
        let map = Map::new(width, height, seed);

        // Count the number of obstacle tiles
        let obstacle_count = map.tiles.iter().filter(|tile| matches!(tile, Tile::Obstacle)).count();
        
        // Verify that at least one obstacle exists but not the entire map
        assert!(obstacle_count > 0, "The map should contain at least one obstacle.");
        assert!(obstacle_count < (width * height) as usize, "The entire map should not be obstacles.");
    }

    /// Test that generating a map with the same seed results in identical maps.
    #[test]
    fn test_map_seed_reproducibility() {
        let width = 10;
        let height = 10;
        let seed = 42;
        
        let map1 = Map::new(width, height, seed);
        let map2 = Map::new(width, height, seed);

        // Ensure that two maps generated with the same seed are identical
        assert_eq!(map1.tiles, map2.tiles, "Maps with the same seed should be identical.");
    }
}
