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
                    // Define appearance of different tile types
                    let (ch, style) = match tile {
                        Tile::Obstacle => ('#', Style::default().fg(Color::Red)),      // Red obstacles
                        Tile::Empty => ('.', Style::default().fg(Color::Gray)),        // Gray empty space
                        Tile::Energy(_) => ('E', Style::default().fg(Color::Yellow)),  // Yellow energy
                        Tile::Mineral(_) => ('M', Style::default().fg(Color::Blue)),   // Blue minerals
                        Tile::ScientificPoint(_) => ('S', Style::default().fg(Color::Green)), // Green scientific points
                    };
                    
                    // Draw the tile at the correct position in the terminal buffer
                    buf.get_mut(area.x + x + x_offset, area.y + y + y_offset)
                        .set_char(ch)
                        .set_style(style);
                }
            }
        }
    }
}