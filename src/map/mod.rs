use noise::{NoiseFn, Value};
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
pub mod map_widget;

const OBSTACLE_THRESHOLD: f64 = 0.4;
const NOISE_SCALE: f64 = 0.2;
const BASE_COUNT_MIN: u32 = 5;
const BASE_COUNT_MAX: u32 = 15;
const BASE_AMOUNT_MIN: u32 = 5000;
const BASE_AMOUNT_MAX: u32 = 20000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Energy {
    pub amount: u32,
    pub is_base: bool, 
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mineral {
    pub amount: u32,
    pub is_base: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScientificPoint {
    pub value: u32,
    pub is_base: bool,
}

// Represents the entire game map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub width: u32,
    pub height: u32,
    tiles: Vec<Tile>,
    pub seed: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Tile {
    Empty,
    Obstacle,
    Energy(Energy),
    Mineral(Mineral),
    ScientificPoint(ScientificPoint),
    Station,
}

impl Map {

    pub fn new(width: u32, height: u32, seed: u64) -> Self {
        let mut tiles = vec![Tile::Empty; (width * height) as usize];
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

        // Generate obstacles using noise
        let obstacle_noise = Value::new(seed as u32);
        for y in 0..height {
            for x in 0..width {
                let noise_value =
                    obstacle_noise.get([x as f64 * NOISE_SCALE, y as f64 * NOISE_SCALE]);
                if noise_value > OBSTACLE_THRESHOLD {
                    let index = (y * width + x) as usize;
                    tiles[index] = Tile::Obstacle;
                }
            }
        }

        let center_x = width / 2;
        let center_y = height / 2;
        let center_index = (center_y * width + center_x) as usize;
        tiles[center_index] = Tile::Empty;

        let find_valid_position =
            |tiles: &Vec<Tile>, width: u32, rng: &mut rand::rngs::StdRng| -> Option<(u32, u32)> {
                for _ in 0..100 {
                    let x = rng.gen_range(0..width);
                    let y = rng.gen_range(0..height);
                    let index = (y * width + x) as usize;
                    if let Tile::Empty = tiles[index] {
                        return Some((x, y));
                    }
                }
                None
            };

        // Generate Energy Bases
        let energy_base_count = rng.gen_range(BASE_COUNT_MIN..=BASE_COUNT_MAX);
        for _ in 0..energy_base_count {
            if let Some((x, y)) = find_valid_position(&tiles, width, &mut rng) {
                let amount = rng.gen_range(BASE_AMOUNT_MIN..=BASE_AMOUNT_MAX);
                let index = (y * width + x) as usize;
                tiles[index] = Tile::Energy(Energy {
                    amount,
                    is_base: true,
                });
            }
        }

        // Generate Mineral Bases
        let mineral_base_count = rng.gen_range(BASE_COUNT_MIN..=BASE_COUNT_MAX);
        for _ in 0..mineral_base_count {
            if let Some((x, y)) = find_valid_position(&tiles, width, &mut rng) {
                let amount = rng.gen_range(BASE_AMOUNT_MIN..=BASE_AMOUNT_MAX);
                let index = (y * width + x) as usize;
                tiles[index] = Tile::Mineral(Mineral {
                    amount,
                    is_base: true,
                });
            }
        }

        // Generate Scientific Points
        let science_base_count = rng.gen_range(BASE_COUNT_MIN..=BASE_COUNT_MAX);
        for _ in 0..science_base_count {
            if let Some((x, y)) = find_valid_position(&tiles, width, &mut rng) {
                let value = rng.gen_range(BASE_AMOUNT_MIN..=BASE_AMOUNT_MAX);
                let index = (y * width + x) as usize;
                tiles[index] = Tile::ScientificPoint(ScientificPoint {
                    value,
                    is_base: true,
                });
            }
        }

        Map {
            width,
            height,
            tiles,
            seed,
        }
    }

    pub fn get_tile(&self, x: u32, y: u32) -> Option<&Tile> {
        if x >= self.width || y >= self.height {
            return None; 
        }
        self.tiles.get((y * self.width + x) as usize)
    }

    pub fn get_tile_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.tiles.get_mut((y * self.width + x) as usize)
    }

    pub fn is_obstacle(&self, x: u32, y: u32) -> bool {
        matches!(self.get_tile(x, y), Some(Tile::Obstacle))
    }

    pub fn has_scientific_point(&self, x: u32, y: u32) -> bool {
        matches!(self.get_tile(x, y), Some(Tile::ScientificPoint(_)))
    }

    pub fn consume_energy(&mut self, x: u32, y: u32, amount: u32) -> Option<u32> {
        if let Some(tile) = self.get_tile_mut(x, y) {
            if let Tile::Energy(energy) = tile {
                let consumed = amount.min(energy.amount);
                energy.amount -= consumed;

                if energy.amount == 0 {
                    *tile = Tile::Empty;
                }

                return Some(consumed);
            }
        }
        None
    }

    pub fn consume_mineral(&mut self, x: u32, y: u32, amount: u32) -> Option<u32> {
        if let Some(tile) = self.get_tile_mut(x, y) {
            if let Tile::Mineral(mineral) = tile {
                let consumed = amount.min(mineral.amount);
                mineral.amount -= consumed;

                if mineral.amount == 0 {
                    *tile = Tile::Empty;
                }

                return Some(consumed);
            }
        }
        None
    }

    pub fn extract_scientific_data(&mut self, x: u32, y: u32) -> Option<u32> {
        if let Some(tile) = self.get_tile_mut(x, y) {
            if let Tile::ScientificPoint(point) = tile {
                let value = point.value;
                *tile = Tile::Empty;
                return Some(value);
            }
        }
        None
    }

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

    pub fn count_resource_bases(&self) -> (u32, u32, u32) {
        let mut energy_bases = 0;
        let mut mineral_bases = 0;
        let mut scientific_bases = 0;

        for tile in &self.tiles {
            match tile {
                Tile::Energy(energy) if energy.is_base => energy_bases += 1,
                Tile::Mineral(mineral) if mineral.is_base => mineral_bases += 1,
                Tile::ScientificPoint(point) if point.is_base => scientific_bases += 1,
                _ => {}
            }
        }

        (energy_bases, mineral_bases, scientific_bases)
    }

    pub fn calculate_total_resources(&self) -> (u32, u32, u32) {
        let mut energy_total = 0;
        let mut mineral_total = 0;
        let mut scientific_total = 0;

        for tile in &self.tiles {
            match tile {
                Tile::Energy(energy) => energy_total += energy.amount,
                Tile::Mineral(mineral) => mineral_total += mineral.amount,
                Tile::ScientificPoint(point) => scientific_total += point.value,
                _ => {}
            }
        }

        (energy_total, mineral_total, scientific_total)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    
    #[test]
    fn test_map_generation() {
        let width = 10;
        let height = 10;
        let seed = 42;
        let map = Map::new(width, height, seed);

        
        assert_eq!(map.width, width, "Map width should be correct.");
        assert_eq!(map.height, height, "Map height should be correct.");
        assert_eq!(
            map.tiles.len(),
            (width * height) as usize,
            "Total number of tiles should match dimensions."
        );
    }

    #[test]
    fn test_obstacles_generation() {
        let width = 10;
        let height = 10;
        let seed = 42;
        let map = Map::new(width, height, seed);

        let obstacle_count = map
            .tiles
            .iter()
            .filter(|tile| matches!(tile, Tile::Obstacle))
            .count();

        assert!(
            obstacle_count > 0,
            "The map should contain at least one obstacle."
        );
        assert!(
            obstacle_count < (width * height) as usize,
            "The entire map should not be obstacles."
        );
    }

    #[test]
    fn test_map_seed_reproducibility() {
        let width = 10;
        let height = 10;
        let seed = 42;

        let map1 = Map::new(width, height, seed);
        let map2 = Map::new(width, height, seed);

        assert_eq!(
            map1.tiles, map2.tiles,
            "Maps with the same seed should be identical."
        );
    }

    #[test]
    fn test_resource_generation() {
        let width = 100;
        let height = 100;
        let seed = 42;
        let map = Map::new(width, height, seed);

        let (energy_count, mineral_count, scientific_count) = map.resource_statistics();

        assert!(energy_count > 0, "The map should contain energy resources");
        assert!(
            mineral_count > 0,
            "The map should contain mineral resources"
        );
        assert!(
            scientific_count > 0,
            "The map should contain scientific points"
        );
    }

    #[test]
    fn test_resource_consumption() {
        let width = 50;
        let height = 50;
        let seed = 42;
        let mut map = Map::new(width, height, seed);

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
            let original_amount = if let Some(Tile::Energy(energy)) = map.get_tile(x, y) {
                energy.amount
            } else {
                0
            };

            let consumed = map.consume_energy(x, y, 10).unwrap_or(0);
            assert!(consumed > 0, "Should consume some energy");
            assert!(consumed <= 10, "Should not consume more than requested");

            let remaining = if let Some(Tile::Energy(energy)) = map.get_tile(x, y) {
                energy.amount
            } else {
                0
            };

            assert_eq!(
                original_amount - consumed,
                remaining,
                "Remaining amount should be correct"
            );
        }

        // Test mineral consumption
        if let Some((x, y)) = mineral_pos {
            let original_amount = if let Some(Tile::Mineral(mineral)) = map.get_tile(x, y) {
                mineral.amount
            } else {
                0
            };

            let consumed = map.consume_mineral(x, y, original_amount).unwrap_or(0);
            assert_eq!(consumed, original_amount, "Should consume all minerals");

            assert!(
                matches!(map.get_tile(x, y), Some(Tile::Empty)),
                "Tile should be empty after consuming all resources"
            );
        }
    }

    /// Test scientific point extraction.
    #[test]
    fn test_scientific_point_extraction() {
        let width = 50;
        let height = 50;
        let seed = 42;
        let mut map = Map::new(width, height, seed);

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

        if let Some((x, y)) = science_pos {
            let has_point = map.has_scientific_point(x, y);
            assert!(has_point, "Should detect scientific point");

            let data = map.extract_scientific_data(x, y);
            assert!(data.is_some(), "Should extract scientific data");

            let still_has_point = map.has_scientific_point(x, y);
            assert!(
                !still_has_point,
                "Scientific point should be gone after extraction"
            );
        }
    }
}
