#[allow(dead_code)]
use crate::map::{Map, Tile};
use crate::robot::Position;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapKnowledge {
    pub version: u64,
    pub explored_tiles: HashMap<(u32, u32), ExploredTile>,
    pub updates_history: Vec<MapUpdate>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploredTile {
    pub tile: Tile,
    pub version: u64,
    pub explorer_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapUpdate {
    pub version: u64,
    pub positions: Vec<Position>,
    pub timestamp: u64,
    pub robot_id: u32,
}

impl MapKnowledge {
    pub fn new() -> Self {
        Self {
            version: 0,
            explored_tiles: HashMap::new(),
            updates_history: Vec::new(),
        }
    }

    pub fn merge_robot_knowledge(
        &mut self,
        robot_tiles: HashMap<(u32, u32), ExploredTile>,
        robot_id: u32,
    ) -> Vec<Position> {
        let mut updated_positions = Vec::new();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for ((x, y), robot_tile) in robot_tiles {
            let position = Position { x, y };

            if let Some(station_tile) = self.explored_tiles.get_mut(&(x, y)) {
                if robot_tile.version > station_tile.version {
                    *station_tile = robot_tile;
                    updated_positions.push(position);
                }
            } else {
                self.explored_tiles.insert((x, y), robot_tile);
                updated_positions.push(position);
            }
        }

        if !updated_positions.is_empty() {
            self.version += 1;

            self.updates_history.push(MapUpdate {
                version: self.version,
                positions: updated_positions.clone(),
                timestamp,
                robot_id,
            });
        }

        updated_positions
    }

    pub fn generate_partial_map(&self, width: u32, height: u32) -> Map {
        let mut map = Map::new(width, height, 0); 

        for ((x, y), explored_tile) in &self.explored_tiles {
            if *x < width && *y < height {
                if let Some(tile) = map.get_tile_mut(*x, *y) {
                    *tile = explored_tile.tile.clone();
                }
            }
        }

        map
    }

    pub fn has_conflicts(&self, other: &MapKnowledge) -> bool {
        for ((x, y), tile) in &self.explored_tiles {
            if let Some(other_tile) = other.explored_tiles.get(&(*x, *y)) {
                if tile.version == other_tile.version && tile.tile != other_tile.tile {
                    return true;
                }
            }
        }
        false
    }

    pub fn resolve_conflicts(&mut self, other: &MapKnowledge) {
        for ((x, y), other_tile) in &other.explored_tiles {
            if let Some(tile) = self.explored_tiles.get_mut(&(*x, *y)) {
                if other_tile.version > tile.version {
                    *tile = other_tile.clone();
                }
            } else {
                self.explored_tiles.insert((*x, *y), other_tile.clone());
            }
        }

        self.version = self.version.max(other.version) + 1;
    }
}
