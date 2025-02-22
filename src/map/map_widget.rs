use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

use crate::map::{Map, Tile};

pub struct MapWidget<'a> {
    map: &'a Map,
}

impl<'a> MapWidget<'a> {
    pub fn new(map: &'a Map) -> Self {
        Self { map }
    }

    // Fonction helper pour dessiner la base
    fn draw_base(&self, x: u32, y: u32, buf: &mut Buffer, area: Rect, x_offset: u16, y_offset: u16) {
        let base_style = Style::default().fg(Color::Blue);
        // Dessiner un grand cercle 3x3
        let base_chars = [
            ('╭', -1, -1), ('─', 0, -1), ('╮', 1, -1),
            ('│', -1, 0), ('O', 0, 0), ('│', 1, 0),
            ('╰', -1, 1), ('─', 0, 1), ('╯', 1, 1),
        ];

        for (ch, dx, dy) in base_chars.iter() {
            let new_x = x as i32 + dx;
            let new_y = y as i32 + dy;
            
            if new_x >= 0 && new_y >= 0 && new_x < self.map.width as i32 && new_y < self.map.height as i32 {
                let buf_x = area.x + (new_x as u16) + x_offset;
                let buf_y = area.y + (new_y as u16) + y_offset;
                
                if buf_x < area.x + area.width && buf_y < area.y + area.height {
                    buf.get_mut(buf_x, buf_y)
                        .set_char(*ch)
                        .set_style(base_style);
                }
            }
        }
    }
}

impl Widget for MapWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let x_offset = 1;
        let y_offset = 1;
        let display_width = area.width.saturating_sub(2);
        let display_height = area.height.saturating_sub(2);

        for y in 0..display_height {
            for x in 0..display_width {
                let map_x = x as u32;
                let map_y = y as u32;
                
                // Ne pas dessiner de tuile si nous sommes dans la zone de la base
                let is_base_area = (map_x as i32 - self.map.base_position.0 as i32).abs() <= 1 &&
                                 (map_y as i32 - self.map.base_position.1 as i32).abs() <= 1;
                
                if !is_base_area {
                    if let Some(tile) = self.map.get_tile(map_x, map_y) {
                        let (ch, style) = match tile {
                            Tile::Obstacle => ('#', Style::default().fg(Color::Red)),
                            Tile::Empty { scientific_interest, .. } => {
                                if *scientific_interest {
                                    ('*', Style::default().fg(Color::Yellow))
                                } else {
                                    ('.', Style::default().fg(Color::Gray))
                                }
                            }
                        };
                        
                        buf.get_mut(area.x + x + x_offset, area.y + y + y_offset)
                            .set_char(ch)
                            .set_style(style);
                    }
                }
            }
        }

        // Dessiner la base après les autres tuiles pour s'assurer qu'elle est au-dessus
        self.draw_base(self.map.base_position.0, self.map.base_position.1, buf, area, x_offset as u16, y_offset as u16);
    }
}