use crate::map::{Map, Tile};
use crate::robot::Position;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Structure pour représenter les informations sur la carte connues par la station
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapKnowledge {
    /// Version actuelle de la carte connue par la station
    pub version: u64,
    /// Tuiles explorées et leur dernière version connue
    pub explored_tiles: HashMap<(u32, u32), ExploredTile>,
    /// Historique des mises à jour de la carte
    pub updates_history: Vec<MapUpdate>,
}

/// Structure pour représenter une tuile explorée
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploredTile {
    /// Contenu de la tuile
    pub tile: Tile,
    /// Version à laquelle cette tuile a été mise à jour pour la dernière fois
    pub version: u64,
    /// Robot qui a exploré cette tuile
    pub explorer_id: Option<u32>,
}

/// Structure pour représenter une mise à jour de la carte
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapUpdate {
    /// Version de la mise à jour
    pub version: u64,
    /// Position des tuiles mises à jour
    pub positions: Vec<Position>,
    /// Horodatage de la mise à jour
    pub timestamp: u64,
    /// Robot qui a effectué la mise à jour
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

    /// Fusionne les connaissances de la carte d'un robot avec celles de la station
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

            // Vérifier si la tuile existe déjà dans la connaissance de la station
            if let Some(station_tile) = self.explored_tiles.get_mut(&(x, y)) {
                // Ne mettre à jour que si la version du robot est plus récente
                if robot_tile.version > station_tile.version {
                    *station_tile = robot_tile;
                    updated_positions.push(position);
                }
            } else {
                // Nouvelle tuile explorée
                self.explored_tiles.insert((x, y), robot_tile);
                updated_positions.push(position);
            }
        }

        // S'il y a des mises à jour, incrémenter la version et enregistrer l'historique
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

    /// Génère une carte partielle basée sur les connaissances actuelles
    pub fn generate_partial_map(&self, width: u32, height: u32) -> Map {
        // Créer une nouvelle carte vide
        let mut map = Map::new(width, height, 0); // Seed 0 car on va remplacer les tuiles

        // Remplacer les tuiles connues
        for ((x, y), explored_tile) in &self.explored_tiles {
            if *x < width && *y < height {
                if let Some(tile) = map.get_tile_mut(*x, *y) {
                    *tile = explored_tile.tile.clone();
                }
            }
        }

        map
    }

    /// Vérifie s'il y a des conflits entre deux versions de la carte
    pub fn has_conflicts(&self, other: &MapKnowledge) -> bool {
        for ((x, y), tile) in &self.explored_tiles {
            if let Some(other_tile) = other.explored_tiles.get(&(*x, *y)) {
                // Conflit si les deux versions ont la même version mais des tuiles différentes
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
