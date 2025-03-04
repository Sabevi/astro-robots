use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::{io, time::Duration};

mod map;
mod robot; // Import the robot module

use map::map_widget::MapWidget;
use robot::{HardwareModule, Position, Robot}; // Import Robot and related types

mod station;
use station::{RobotType, Station};

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

    // Initialize station
    let station_position = Position {
        x: MAP_WIDTH / 2,
        y: MAP_HEIGHT / 2,
    };
    let mut station = Station::new(station_position);

    // Initialize robots with specific positions and hardware modules
    let mut robots = vec![
        Robot::new(
            Position { x: 10, y: 10 },
            vec![HardwareModule::TerrainScanner {
                efficiency: 0.8,
                range: 15,
            }],
        ),
        Robot::new(
            Position { x: 20, y: 20 },
            vec![HardwareModule::DeepDrill { mining_speed: 1.5 }],
        ),
    ];

    if let Some(robot) = station.create_robot(RobotType::Explorer) {
        robots.push(robot);
    }

    if let Some(robot) = station.create_robot(RobotType::EnergyCollector) {
        robots.push(robot);
    }

    // Main rendering loop
    loop {
        terminal.draw(|f| {
            // Create vertical layout with map and info sections
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Min(3),
                        Constraint::Length(4), // Increased to accommodate additional info line
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // Render map section with border
            let map_block = Block::default().title("Robots Swarm").borders(Borders::ALL);

            let map_widget = MapWidget::new(&map);
            f.render_widget(map_block.clone(), chunks[0]);
            f.render_widget(map_widget, chunks[0].inner(&Default::default()));

            // Render information bar with controls and map data
            let info_block = Block::default().title("Commands").borders(Borders::ALL);

            // Get resource statistics
            let (energy_bases, mineral_bases, scientific_bases) = map.count_resource_bases();
            let (energy_total, mineral_total, scientific_total) = map.calculate_total_resources();

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
                    Span::raw("Energy Bases ("),
                    Span::styled("⚡", Style::default().fg(Color::Yellow)),
                    Span::raw("): "),
                    Span::styled(energy_bases.to_string(), Style::default().fg(Color::Yellow)),
                    Span::raw(" (Total: "),
                    Span::styled(energy_total.to_string(), Style::default().fg(Color::Yellow)),
                    Span::raw(") | Mineral Bases ("),
                    Span::styled("♦", Style::default().fg(Color::Blue)),
                    Span::raw("): "),
                    Span::styled(mineral_bases.to_string(), Style::default().fg(Color::Blue)),
                    Span::raw(" (Total: "),
                    Span::styled(mineral_total.to_string(), Style::default().fg(Color::Blue)),
                    Span::raw(") | Scientific Bases ("),
                    Span::styled("★", Style::default().fg(Color::Green)),
                    Span::raw("): "),
                    Span::styled(
                        scientific_bases.to_string(),
                        Style::default().fg(Color::Green),
                    ),
                    Span::raw(" (Total: "),
                    Span::styled(
                        scientific_total.to_string(),
                        Style::default().fg(Color::Green),
                    ),
                    Span::raw(")"),
                ]),
            ];

            let info = Paragraph::new(info_text)
                .block(info_block)
                .style(Style::default());

            f.render_widget(info, chunks[1]);
        })?;

        station.update(&map);

        // Handle user input
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('r') => {
                        let random_seed = rand::random::<u64>();
                        map = map::Map::new(MAP_WIDTH, MAP_HEIGHT, random_seed);
                        robots
                            .iter_mut()
                            .for_each(|robot| robot.start_exploring(Position { x: 0, y: 0 }));
                    }
                    _ => {}
                }
            }
        }

        // Update robots' positions
        for robot in &mut robots {
            robot.update(&map);
        }
    }

    // Restore terminal state
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
