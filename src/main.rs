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
mod simulation;

use map::map_widget::MapWidget;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let random_seed = rand::random::<u64>();
    let mut map = map::Map::new(200, 100, random_seed);
    
    loop {
        terminal.draw(|f| {
            // Créer un layout vertical avec une zone pour la carte et une pour les infos
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),     // Pour la carte
                    Constraint::Length(3),  // Pour la barre d'information
                ].as_ref())
                .split(f.size());
            
            // Zone de la carte
            let map_block = Block::default()
                .title("EREEA Simulation")
                .borders(Borders::ALL);
            
            let map_widget = MapWidget::new(&map);
            f.render_widget(map_block.clone(), chunks[0]);
            f.render_widget(map_widget, chunks[0].inner(&Default::default()));

            // Barre d'information en bas
            let info_block = Block::default()
                .title("Commandes")
                .borders(Borders::ALL);

            let info_text = vec![
                Line::from(vec![
                    Span::raw("Appuyez sur "),
                    Span::styled("'r'", Style::default().fg(Color::Yellow)),
                    Span::raw(" pour régénérer la carte | "),
                    Span::styled("'q'", Style::default().fg(Color::Yellow)),
                    Span::raw(" pour quitter | "),
                    Span::raw("Base position: "),
                    Span::styled(
                        format!("({}, {})", map.base_position.0, map.base_position.1),
                        Style::default().fg(Color::Blue)
                    ),
                    Span::raw(" | Seed: "),
                    Span::styled(map.seed.to_string(), Style::default().fg(Color::Cyan)),
                ]),
            ];

            let info = Paragraph::new(info_text)
                .block(info_block)
                .style(Style::default());

            f.render_widget(info, chunks[1]);
        })?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('r') => {
                        let random_seed = rand::random::<u64>();
                        map = map::Map::new(200, 100, random_seed);
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