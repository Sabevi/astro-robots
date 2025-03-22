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
use std::{
    io,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

mod map;
mod robot;

use map::map_widget::MapWidget;
use robot::{HardwareModule, Position, Robot};

mod station;
use station::{RobotType, Station};

const MAP_WIDTH: u32 = 200;
const MAP_HEIGHT: u32 = 100;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let random_seed = rand::random::<u64>();
    let map = Arc::new(Mutex::new(map::Map::new(
        MAP_WIDTH,
        MAP_HEIGHT,
        random_seed,
    )));

    let station = Arc::new(Mutex::new(Station::new(&mut map.lock().unwrap())));

    let robots = Arc::new(Mutex::new(vec![
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
    ]));

    let running = Arc::new(Mutex::new(true));
    {
        let running_clone = Arc::clone(&running);
        let robots_clone = Arc::clone(&robots);
        let map_clone = Arc::clone(&map);

        thread::spawn(move || {
            while *running_clone.lock().unwrap() {
                thread::sleep(Duration::from_millis(100));
                let mut robots = robots_clone.lock().unwrap();
                let map = map_clone.lock().unwrap();
                for robot in robots.iter_mut() {
                    robot.move_randomly(&map);
                }
            }
        });
    }

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(5)].as_ref())
                .split(f.size());

            // Get map data once per frame
            let map_lock = map.lock().unwrap();
            let robots_lock = robots.lock().unwrap();

            // Render map with robots
            let map_block = Block::default().title("Robots Swarm").borders(Borders::ALL);
            let map_widget = MapWidget::new(&map_lock, &robots_lock);
            f.render_widget(map_block.clone(), chunks[0]);
            f.render_widget(map_widget, chunks[0].inner(&Default::default()));

            // Render info panel
            let (energy_bases, mineral_bases, scientific_bases) = map_lock.count_resource_bases();
            let (energy_total, mineral_total, scientific_total) =
                map_lock.calculate_total_resources();

            let station_lock = station.lock().unwrap();

            let info_text = vec![
                Line::from(vec![
                    Span::raw("Press "),
                    Span::styled("'r'", Style::default().fg(Color::Yellow)),
                    Span::raw(" to regenerate map | "),
                    Span::styled("'c'", Style::default().fg(Color::Yellow)),
                    Span::raw(" to create robot | "),
                    Span::styled("'q'", Style::default().fg(Color::Yellow)),
                    Span::raw(" to quit | Seed: "),
                    Span::styled(map_lock.seed.to_string(), Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::styled("⚡", Style::default().fg(Color::Yellow)),
                    Span::raw(format!(" {energy_bases} ({energy_total}) | ")),
                    Span::styled("♦", Style::default().fg(Color::Blue)),
                    Span::raw(format!(" {mineral_bases} ({mineral_total}) | ")),
                    Span::styled("★", Style::default().fg(Color::Green)),
                    Span::raw(format!(" {scientific_bases} ({scientific_total})")),
                ]),
                Line::from(vec![
                    Span::raw("📦 Station - "),
                    Span::raw("Energy: "),
                    Span::styled(
                        station_lock.resources.energy.to_string(),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::raw(" | Minerals: "),
                    Span::styled(
                        station_lock.resources.minerals.to_string(),
                        Style::default().fg(Color::Blue),
                    ),
                    Span::raw(" | Scientific Data: "),
                    Span::styled(
                        station_lock.resources.scientific_data.to_string(),
                        Style::default().fg(Color::Green),
                    ),
                ]),
            ];

            let info_block = Block::default().title("Commands").borders(Borders::ALL);
            let info = Paragraph::new(info_text).block(info_block);
            f.render_widget(info, chunks[1]);
        })?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('r') => {
                        let mut map_lock = map.lock().unwrap();
                        let mut robots_lock = robots.lock().unwrap();

                        *map_lock = map::Map::new(MAP_WIDTH, MAP_HEIGHT, rand::random());
                        robots_lock.iter_mut().for_each(|robot| {
                            robot.position = Position { x: 0, y: 0 };
                        });

                        Station::new(&mut map_lock);
                    }
                    KeyCode::Char('c') => {
                        let mut station_lock = station.lock().unwrap();
                        let mut robots_lock = robots.lock().unwrap();

                        if let Some(new_robot) = station_lock.create_robot(RobotType::Explorer) {
                            robots_lock.push(new_robot);
                        }
                    }

                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
