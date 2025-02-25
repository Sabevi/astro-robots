// ========================= IMPORTS ========================= //
use noise::{NoiseFn, Value};
use serde::{Serialize, Deserialize};
pub mod map_widget;  

// ========================= CONSTANTS ========================= //
const OBSTACLE_THRESHOLD: f64 = 0.4; 
const NOISE_SCALE: f64 = 0.2;
const ENERGY_THRESHOLD: f64 = -0.2;      // Valeur de bruit pour l'énergie
const MINERAL_THRESHOLD: f64 = 0.0;      // Valeur de bruit pour les minéraux
const SCIENTIFIC_THRESHOLD: f64 = 0.2;   // Valeur de bruit pour les points scientifiques
const RESOURCE_NOISE_SCALE: f64 = 0.1;   // Échelle différente pour les ressources
const MIN_RESOURCE_AMOUNT: u32 = 50;     // Quantité minimale
const MAX_RESOURCE_AMOUNT: u32 = 200;    // Quantité maximale

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
    Empty,                // Walkable space with no obstacle
    Obstacle,             // Non-walkable space
    Energy(u32),          // Energy resource with amount
    Mineral(u32),         // Mineral resource with amount
    ScientificPoint(u32), // Scientific interest point with value
}


// ========================= MAP IMPLEMENTATION ========================= //
impl Map {
    /// Creates a new map with the given dimensions and seed.
    pub fn new(width: u32, height: u32, seed: u64) -> Self {
        let mut tiles = Vec::with_capacity((width * height) as usize);
        let obstacle_noise = Value::new(seed as u32);
        let resource_noise = Value::new((seed.wrapping_add(1)) as u32);
        
        // Generate the map tiles
        for y in 0..height {
            for x in 0..width {
                let obstacle_value = obstacle_noise.get([x as f64 * NOISE_SCALE, y as f64 * NOISE_SCALE]);
                let resource_value = resource_noise.get([x as f64 * RESOURCE_NOISE_SCALE, y as f64 * RESOURCE_NOISE_SCALE]);
                
                // Calculate resource amount based on noise value (more extreme = more resources)
                let amount = MIN_RESOURCE_AMOUNT + 
                    (((resource_value.abs() * 2.0).min(1.0)) * (MAX_RESOURCE_AMOUNT - MIN_RESOURCE_AMOUNT) as f64) as u32;
                
                // Determine tile type
                let tile = if obstacle_value > OBSTACLE_THRESHOLD {
                    Tile::Obstacle
                } else if resource_value < ENERGY_THRESHOLD {
                    Tile::Energy(amount)
                } else if resource_value < MINERAL_THRESHOLD {
                    Tile::Mineral(amount)
                } else if resource_value < SCIENTIFIC_THRESHOLD {
                    Tile::ScientificPoint(amount)
                } else {
                    Tile::Empty
                };
                
                tiles.push(tile);
            }
        }
        
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
    
    /// Attempts to consume an energy resource at the given position.
    /// Returns the amount consumed or None if no energy resource exists there.
    pub fn consume_energy(&mut self, x: u32, y: u32, amount: u32) -> Option<u32> {
        self.consume_resource(x, y, |tile| {
            if let Tile::Energy(energy_amount) = tile {
                Some(*energy_amount)
            } else {
                None
            }
        }, amount)
    }
    
    /// Attempts to consume a mineral resource at the given position.
    /// Returns the amount consumed or None if no mineral resource exists there.
    pub fn consume_mineral(&mut self, x: u32, y: u32, amount: u32) -> Option<u32> {
        self.consume_resource(x, y, |tile| {
            if let Tile::Mineral(mineral_amount) = tile {
                Some(*mineral_amount)
            } else {
                None
            }
        }, amount)
    }
    
    /// Returns true if the given position contains a scientific interest point.
    pub fn has_scientific_point(&self, x: u32, y: u32) -> bool {
        matches!(self.get_tile(x, y), Some(Tile::ScientificPoint(_)))
    }
    
    /// Extracts data from a scientific point. Returns the value or None if no scientific point exists there.
    pub fn extract_scientific_data(&mut self, x: u32, y: u32) -> Option<u32> {
        if let Some(tile) = self.get_tile_mut(x, y) {
            if let Tile::ScientificPoint(value) = *tile {
                *tile = Tile::Empty;
                return Some(value);
            }
        }
        None
    }
    
    // Helper method for resource consumption
    fn consume_resource<F>(&mut self, x: u32, y: u32, extractor: F, amount: u32) -> Option<u32>
    where
        F: Fn(&Tile) -> Option<u32>,
    {
        if let Some(tile) = self.get_tile(x, y) {
            if let Some(available) = extractor(tile) {
                let consumed = amount.min(available);
                if let Some(tile_mut) = self.get_tile_mut(x, y) {
                    match *tile_mut {
                        Tile::Energy(ref mut energy_amount) => {
                            *energy_amount -= consumed;
                            if *energy_amount == 0 {
                                *tile_mut = Tile::Empty;
                            }
                        },
                        Tile::Mineral(ref mut mineral_amount) => {
                            *mineral_amount -= consumed;
                            if *mineral_amount == 0 {
                                *tile_mut = Tile::Empty;
                            }
                        },
                        _ => return None,
                    }
                    return Some(consumed);
                }
            }
        }
        None
    }
    
    /// Get resource counts on the map
    pub fn resource_statistics(&self) -> (u32, u32, u32) {
        let mut energy_count = 0;
        let mut mineral_count = 0;
        let mut scientific_count = 0;
        
        for tile in &self.tiles {
            match tile {
                Tile::Energy(_) => energy_count += 1,
                Tile::Mineral(_) => mineral_count += 1,
                Tile::ScientificPoint(_) => scientific_count += 1,
                _ => {}
            }
        }
        
        (energy_count, mineral_count, scientific_count)
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
    
    /// Test that resources are generated.
    #[test]
    fn test_resource_generation() {
        let width = 100;
        let height = 100;
        let seed = 42;
        let map = Map::new(width, height, seed);
        
        let (energy_count, mineral_count, scientific_count) = map.resource_statistics();
        
        // Check that some resources are generated
        assert!(energy_count > 0, "The map should contain energy resources");
        assert!(mineral_count > 0, "The map should contain mineral resources");
        assert!(scientific_count > 0, "The map should contain scientific points");
    }
    
    /// Test resource consumption.
    #[test]
    fn test_resource_consumption() {
        let width = 50;
        let height = 50;
        let seed = 42;
        let mut map = Map::new(width, height, seed);
        
        // Find energy and mineral resources
        let mut energy_pos = None;
        let mut mineral_pos = None;
        
        for y in 0..height {
            for x in 0..width {
                match map.get_tile(x, y) {
                    Some(Tile::Energy(_)) if energy_pos.is_none() => energy_pos = Some((x, y)),
                    Some(Tile::Mineral(_)) if mineral_pos.is_none() => mineral_pos = Some((x, y)),
                    _ => continue,
                }
                if energy_pos.is_some() && mineral_pos.is_some() {
                    break;
                }
            }
        }
        
        // Test energy consumption
        if let Some((x, y)) = energy_pos {
            let original_amount = if let Some(Tile::Energy(amount)) = map.get_tile(x, y) {
                *amount
            } else {
                0
            };
            
            let consumed = map.consume_energy(x, y, 10).unwrap_or(0);
            assert!(consumed > 0, "Should consume some energy");
            assert!(consumed <= 10, "Should not consume more than requested");
            
            let remaining = if let Some(Tile::Energy(amount)) = map.get_tile(x, y) {
                *amount
            } else {
                0
            };
            
            assert_eq!(original_amount - consumed, remaining, "Remaining amount should be correct");
        }
        
        // Test mineral consumption
        if let Some((x, y)) = mineral_pos {
            let original_amount = if let Some(Tile::Mineral(amount)) = map.get_tile(x, y) {
                *amount
            } else {
                0
            };
            
            let consumed = map.consume_mineral(x, y, original_amount).unwrap_or(0);
            assert_eq!(consumed, original_amount, "Should consume all minerals");
            
            // After consuming all, the tile should be empty
            assert!(matches!(map.get_tile(x, y), Some(Tile::Empty)), "Tile should be empty after consuming all resources");
        }
    }
    
    /// Test scientific point extraction.
    #[test]
    fn test_scientific_point_extraction() {
        let width = 50;
        let height = 50;
        let seed = 42;
        let mut map = Map::new(width, height, seed);
        
        // Find a scientific point
        let mut science_pos = None;
        for y in 0..height {
            for x in 0..width {
                if matches!(map.get_tile(x, y), Some(Tile::ScientificPoint(_))) {
                    science_pos = Some((x, y));
                    break;
                }
            }
            if science_pos.is_some() {
                break;
            }
        }
        
        // Test data extraction
        if let Some((x, y)) = science_pos {
            let has_point = map.has_scientific_point(x, y);
            assert!(has_point, "Should detect scientific point");
            
            let data = map.extract_scientific_data(x, y);
            assert!(data.is_some(), "Should extract scientific data");
            
            let still_has_point = map.has_scientific_point(x, y);
            assert!(!still_has_point, "Scientific point should be gone after extraction");
        }
    }
}