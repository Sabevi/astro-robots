use noise::{NoiseFn, Perlin};
use rand::Rng;
use serde::{Serialize, Deserialize};

mod noise_generator;
pub mod map_widget;  // Ajout de cette ligne

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub width: u32,
    pub height: u32,
    pub base_position: (u32, u32),
    tiles: Vec<Tile>,
    pub seed: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Tile {
    Empty {
        energy: f32,
        minerals: f32,
        scientific_interest: bool,
    },
    Obstacle,
}

impl Map {
    pub fn new(width: u32, height: u32, seed: u64) -> Self {
        let mut tiles = Vec::with_capacity((width * height) as usize);
        let perlin = Perlin::new(seed as u32);
        let mut rng = rand::thread_rng();

        // Générer les tuiles
        for y in 0..height {
            for x in 0..width {
                let noise_value = perlin.get([x as f64 / 10.0, y as f64 / 10.0]);
                
                let tile = if noise_value > 0.5 {
                    Tile::Obstacle
                } else {
                    Tile::Empty {
                        energy: rng.gen_range(0.0..1.0),
                        minerals: rng.gen_range(0.0..1.0),
                        scientific_interest: noise_value > 0.8,
                    }
                };
                
                tiles.push(tile);
            }
        }

        // Trouver une position valide pour la base (sur une case vide)
        let mut base_x = 0;
        let mut base_y = 0;
        let mut attempts = 0;
        let max_attempts = 1000;

        'search: loop {
            if attempts >= max_attempts {
                break;
            }
            
            let x = rng.gen_range(0..width);
            let y = rng.gen_range(0..height);
            let index = (y * width + x) as usize;

            if let Tile::Empty { .. } = &tiles[index] {
                base_x = x;
                base_y = y;
                break 'search;
            }
            
            attempts += 1;
        }

        Map {
            width,
            height,
            base_position: (base_x, base_y),
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
}