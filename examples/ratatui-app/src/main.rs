//! Minimal Ratatui TUI application used as the Rusk terminal reference app.
//!
//! It draws a single full-screen view and reacts to keys, which is enough to
//! confirm on the Raspberry Pi that a TUI renders correctly on the Linux
//! virtual console with no desktop environment, compositor, or WebKitGTK.
//!
//! Keys:
//!   SPACE / ENTER  increment the counter ("ping the backend")
//!   q / ESC        quit
//!
//! `q` is the application-level exit. In a real kiosk the systemd unit decides
//! the restart policy; here exiting lets you observe the process lifecycle.

use std::io::{self, stdout, Stdout};
use std::time::Duration;

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;

#[derive(Default)]
struct App {
    pings: u64,
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    // Always restore the terminal, even if the loop returned an error.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    let mut app = App::default();

    loop {
        terminal.draw(|frame| draw(frame, &app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char(' ') | KeyCode::Enter => app.pings += 1,
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn draw(frame: &mut Frame, app: &App) {
    let text = vec![
        Line::from("Rusk TUI example").bold(),
        Line::from(""),
        Line::from(format!("pings: {}", app.pings)),
        Line::from(""),
        Line::from("SPACE / ENTER to ping, q or ESC to quit.").dim(),
    ];

    let block = Block::default().borders(Borders::ALL).title(" rusk ");
    let paragraph = Paragraph::new(text).block(block).centered();
    frame.render_widget(paragraph, frame.area());
}
