use std::{error::Error, io};

use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
    Frame, Terminal,
};

use widgets::{
    event::{Config, Event, Events},
    widgets::{
        separated_list::{ItemDisplay, WindowType},
        ListItem, ListState, SeparatedList,
    },
};

static WORDS: &str = include_str!("wordlist.txt");

fn words() -> Vec<&'static str> {
    WORDS.trim_end().split('\n').collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::with_config(Config::without_ticker());

    let mut state = ListState::new(words().len());
    //state.select(2);

    loop {
        let mstate = &mut state;
        let _ = terminal.draw(|f| draw(mstate, f));

        match events.next()? {
            Event::Input(Key::Char(c)) if c == 'j' => {
                state.cycle_next();
            }
            Event::Input(Key::Char(c)) if c == 'k' => {
                state.cycle_prev();
            }
            Event::Input(Key::Char(_)) => {
                break;
            }
            _ => {}
        };
    }
    Ok(())
}

fn draw<B: Backend>(state: &mut ListState, f: &mut Frame<B>) {
    let size = f.size();
    let layout = Layout::default()
        .margin(2)
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(50),
        ]);
    let splits = layout.split(size);
    let area = splits[0];
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(30), Constraint::Length(area.height - 10)]);
    let area = layout.split(area)[0];

    let bstyle = Style::default().fg(Color::Green);
    let bounds = Block::default()
        .borders(Borders::ALL)
        .title("Simple wrapper")
        .border_type(BorderType::Thick)
        .style(bstyle);

    let items = words().into_iter().map(ListItem::new).collect();

    let sstyle = Style::default().bg(Color::Blue).fg(Color::White);
    //let dstyle = Style::default().bg(Color::LightYellow).fg(Color::White);
    let dstyle = Style::default().bg(Color::Black).fg(Color::White);

    let stupid = SeparatedList::new()
        .block(bounds)
        .default_style(dstyle)
        .selected_style(sstyle)
        .items(items)
        .item_display(ItemDisplay::Separated);

    f.render_stateful_widget(stupid, area, state);
}
