use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::map::{Map, Tile};

// Widget responsible for rendering the map in the terminal
pub struct MapWidget<'a> {
    map: &'a Map, // Reference to the map data
}

impl<'a> MapWidget<'a> {
    // Constructor to create a new MapWidget
    pub fn new(map: &'a Map) -> Self {
        Self { map }
    }    
}

// Constants for resource intensity calculation
const MIN_AMOUNT: u32 = 50;  // Minimum resource amount
const MAX_AMOUNT: u32 = 200; // Maximum resource amount

impl Widget for MapWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let x_offset = 1;
        let y_offset = 1;

        // Adjust display size to fit within both terminal area and map dimensions
        let display_width = area.width.saturating_sub(2).min(self.map.width as u16);
        let display_height = area.height.saturating_sub(2).min(self.map.height as u16);

        // Iterate over the visible portion of the map and render each tile
        for y in 0..display_height {
            for x in 0..display_width {
                let map_x = x as u32;
                let map_y = y as u32;
                
                if let Some(tile) = self.map.get_tile(map_x, map_y) {
                    match tile {
                        Tile::Obstacle => {
                            buf.get_mut(area.x + x + x_offset, area.y + y + y_offset)
                                .set_char('#')
                                .set_style(Style::default().fg(Color::Red));
                        },
                        Tile::Empty => {
                            buf.get_mut(area.x + x + x_offset, area.y + y + y_offset)
                                .set_char('.')
                                .set_style(Style::default().fg(Color::Gray));
                        },
                        Tile::Energy(energy) => {
                            // Special display for energy bases
                            if energy.is_base {
                                buf.get_mut(area.x + x + x_offset, area.y + y + y_offset)
                                    .set_char('⚡')
                                    .set_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));
                            } else {
                                let intensity = calculate_color_intensity(energy.amount);
                                buf.get_mut(area.x + x + x_offset, area.y + y + y_offset)
                                    .set_char('⚡')
                                    .set_style(Style::default().fg(Color::Rgb(255, intensity, 0)));
                            }
                        },
                        Tile::Mineral(mineral) => {
                            // Special display for mineral bases
                            if mineral.is_base {
                                buf.get_mut(area.x + x + x_offset, area.y + y + y_offset)
                                    .set_char('♦')
                                    .set_style(Style::default().fg(Color::Blue).bg(Color::DarkGray));
                            } else {
                                let intensity = calculate_color_intensity(mineral.amount);
                                buf.get_mut(area.x + x + x_offset, area.y + y + y_offset)
                                    .set_char('♦')
                                    .set_style(Style::default().fg(Color::Rgb(0, intensity, 255)));
                            }
                        },
                        Tile::ScientificPoint(point) => {
                            // Special display for scientific bases
                            if point.is_base {
                                buf.get_mut(area.x + x + x_offset, area.y + y + y_offset)
                                    .set_char('★')
                                    .set_style(Style::default().fg(Color::Green).bg(Color::DarkGray));
                            } else {
                                let intensity = calculate_color_intensity(point.value);
                                buf.get_mut(area.x + x + x_offset, area.y + y + y_offset)
                                    .set_char('★')
                                    .set_style(Style::default().fg(Color::Rgb(0, 255, intensity)));
                            }
                        },
                    }
                }
            }
        }
    }
}

// Fonction utilitaire pour calculer l'intensité de la couleur basée sur la quantité
fn calculate_color_intensity(amount: u32) -> u8 {
    // Normaliser la quantité entre MIN_AMOUNT et MAX_AMOUNT
    let normalized = (amount.saturating_sub(MIN_AMOUNT)) as f32 / (MAX_AMOUNT.saturating_sub(MIN_AMOUNT)) as f32;
    // Convertir en intensité de couleur (100-255 pour rester visible)
    (100.0 + normalized.clamp(0.0, 1.0) * 155.0) as u8
}