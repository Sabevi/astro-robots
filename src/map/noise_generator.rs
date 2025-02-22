use noise::{NoiseFn, Perlin};

pub struct NoiseGenerator {
    perlin: Perlin,
    scale: f64,
}

impl NoiseGenerator {
    pub fn new(seed: u32, scale: f64) -> Self {
        Self {
            perlin: Perlin::new(seed),
            scale,
        }
    }

    pub fn get(&self, x: u32, y: u32) -> f64 {
        self.perlin.get([x as f64 * self.scale, y as f64 * self.scale])
    }
}