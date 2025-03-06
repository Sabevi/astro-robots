use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::{map::{Map, Tile}, robot::Robot};

pub struct MapWidget<'a> {
    map: &'a Map,
    robots: &'a Vec<Robot>,
}

impl<'a> MapWidget<'a> {
    pub fn new(map: &'a Map, robots: &'a Vec<Robot>) -> Self {
        Self { map, robots }
    }
}

const MIN_AMOUNT: u32 = 50;
const MAX_AMOUNT: u32 = 200;

impl Widget for MapWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Get the actually renderable area (intersection with buffer bounds)
        let render_area = area.intersection(buf.area);
        let _map_width = self.map.width as u16;
        let _map_height = self.map.height as u16;

        // Render map tiles with proper bounds checking
        for y in 0..render_area.height {
            for x in 0..render_area.width {
                // Convert to map coordinates
                let map_x = x as u32;
                let map_y = y as u32;
                
                // Calculate buffer position
                let buf_x = render_area.x + x;
                let buf_y = render_area.y + y;

                // Skip out-of-bounds positions
                if buf_x >= buf.area.width || buf_y >= buf.area.height {
                    continue;
                }

                if let Some(tile) = self.map.get_tile(map_x, map_y) {
                    let cell = buf.get_mut(buf_x, buf_y);
                    match tile {
                        Tile::Obstacle => {
                            cell.set_char('#')
                                .set_style(Style::default().fg(Color::Red));
                        },
                        Tile::Empty => {
                            cell.set_char('.')
                                .set_style(Style::default().fg(Color::Gray));
                        },
                        Tile::Energy(energy) => {
                            if energy.is_base {
                                cell.set_char('⚡')
                                    .set_style(Style::default()
                                        .fg(Color::Yellow)
                                        .bg(Color::DarkGray));
                            } else {
                                let intensity = calculate_color_intensity(energy.amount);
                                cell.set_char('⚡')
                                    .set_style(Style::default()
                                        .fg(Color::Rgb(255, intensity, 0)));
                            }
                        },
                        Tile::Mineral(mineral) => {
                            if mineral.is_base {
                                cell.set_char('♦')
                                    .set_style(Style::default()
                                        .fg(Color::Blue)
                                        .bg(Color::DarkGray));
                            } else {
                                let intensity = calculate_color_intensity(mineral.amount);
                                cell.set_char('♦')
                                    .set_style(Style::default()
                                        .fg(Color::Rgb(0, intensity, 255)));
                            }
                        },
                        Tile::ScientificPoint(point) => {
                            if point.is_base {
                                cell.set_char('★')
                                    .set_style(Style::default()
                                        .fg(Color::Green)
                                        .bg(Color::DarkGray));
                            } else {
                                let intensity = calculate_color_intensity(point.value);
                                cell.set_char('★')
                                    .set_style(Style::default()
                                        .fg(Color::Rgb(0, 255, intensity)));
                            }
                        },
                    }
                }
            }
        }

        // Render robots with strict bounds checking
        for robot in self.robots {
            // Clamp robot positions to map boundaries
            let x = robot.position.x.clamp(0, self.map.width - 1) as u16;
            let y = robot.position.y.clamp(0, self.map.height - 1) as u16;
            
            // Convert to buffer coordinates
            let buf_x = render_area.x + x;
            let buf_y = render_area.y + y;

            // Final safety check before buffer access
            if buf_x < buf.area.width && buf_y < buf.area.height {
                buf.get_mut(buf_x, buf_y)
                    .set_char('R')
                    .set_style(Style::default().fg(Color::Magenta));
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