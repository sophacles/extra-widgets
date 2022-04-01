use std::{error::Error, io};

use termion::{
    input::{MouseTerminal, TermRead},
    raw::IntoRawMode,
    screen::AlternateScreen,
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame, Terminal,
};

use widgets::widgets::Simple;

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let msg = String::from("hello world");
    let _ = terminal.draw(move |f| draw(msg, f));
    let stdin = io::stdin();
    for evt in stdin.keys() {
        if let Ok(_) = evt {
            return Ok(());
        }
    }
    Ok(())
}

fn draw<B: Backend>(msg: String, f: &mut Frame<B>) {
    let size = f.size();
    let layout = Layout::default()
        .margin(2)
        .constraints([Constraint::Percentage(100)]);
    let chunks = layout.split(size);
    let bstyle = Style::default().bg(Color::Blue).fg(Color::White);
    let bounds = Block::default()
        .borders(Borders::ALL)
        .title("Simple wrapper")
        //.border_type(BorderType::Thick)
        .style(bstyle);
    let sstyle = Style::default().fg(Color::Red);
    let stupid = Simple::new().block(bounds).style(sstyle).msg(&msg);

    f.render_widget(stupid, chunks[0]);
}
