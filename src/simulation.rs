use crate::map::Map;

pub struct Simulation {
    map: Map,
}

impl Simulation {
    pub fn new(map: Map) -> Self {
        Self { map }
    }

    pub fn run(&mut self) {
        // Logique de simulation
    }
}