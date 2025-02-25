use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    style::{Style, Color},
    text::{Line, Span},
    Terminal,
};
use std::{io, time::Duration};

mod map;

use map::map_widget::MapWidget;

// Define fixed dimensions for the simulation map
const MAP_WIDTH: u32 = 200;
const MAP_HEIGHT: u32 = 100;

fn main() -> Result<()> {
    // Initialize terminal in raw mode and alternate screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize map with random seed
    let random_seed = rand::random::<u64>();
    let mut map = map::Map::new(MAP_WIDTH, MAP_HEIGHT, random_seed);
    
    // Main rendering loop
    loop {
        terminal.draw(|f| {
            // Create vertical layout with map and info sections
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),
                    Constraint::Length(4), // Increased to accommodate additional info line
                ].as_ref())
                .split(f.size());
            
            // Render map section with border
            let map_block = Block::default()
                .title("Robots swarm")
                .borders(Borders::ALL);
            
            let map_widget = MapWidget::new(&map);
            f.render_widget(map_block.clone(), chunks[0]);
            f.render_widget(map_widget, chunks[0].inner(&Default::default()));

            // Render information bar with controls and map data
            let info_block = Block::default()
                .title("Commands")
                .borders(Borders::ALL);

            // Get resource statistics
            let (energy_count, mineral_count, scientific_count) = map.resource_statistics();
            
            let info_text = vec![
                Line::from(vec![
                    Span::raw("Press "),
                    Span::styled("'r'", Style::default().fg(Color::Yellow)),
                    Span::raw(" to regenerate map | "),
                    Span::styled("'q'", Style::default().fg(Color::Yellow)),
                    Span::raw(" to quit "),
                    Span::raw(" | Seed: "),
                    Span::styled(map.seed.to_string(), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::raw("Resources: Energy ("),
                    Span::styled("E", Style::default().fg(Color::Yellow)),
                    Span::raw("): "),
                    Span::styled(energy_count.to_string(), Style::default().fg(Color::Yellow)),
                    Span::raw(" | Minerals ("),
                    Span::styled("M", Style::default().fg(Color::Blue)),
                    Span::raw("): "),
                    Span::styled(mineral_count.to_string(), Style::default().fg(Color::Blue)),
                    Span::raw(" | Scientific points ("),
                    Span::styled("S", Style::default().fg(Color::Green)),
                    Span::raw("): "),
                    Span::styled(scientific_count.to_string(), Style::default().fg(Color::Green)),
                ]),
            ];

            let info = Paragraph::new(info_text)
                .block(info_block)
                .style(Style::default());

            f.render_widget(info, chunks[1]);
        })?;

        // Handle user input
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('r') => {
                        let random_seed = rand::random::<u64>();
                        map = map::Map::new(MAP_WIDTH, MAP_HEIGHT, random_seed);
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal state
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}