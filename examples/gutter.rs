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

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::with_config(Config::without_ticker());

    let mut state = ListState::new(8);
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

    let style6 = Style::default().fg(Color::Red).bg(Color::Green);
    let items = vec![
        ListItem::new("0:one"),                             // f:0, l:0   | f:1, l:1
        ListItem::new("1:two\n.\n."),                       // f:1, l:3   | f:3, l:5
        ListItem::new("2:three\n."),                        // f:4, l:5   | f:7, l:8
        ListItem::new("3:four"),                            // f:6, l:6   | f:10, l:10
        ListItem::new("4:five\n."),                         // f:7, l:8   | f:12, l:13
        ListItem::new("5:six"),                             // f:9, l:9   | f:15, l:15
        ListItem::new("6:seven\n...and 1/2").style(style6), // f:10, l:11 | f:17, l:18
        ListItem::new("7:eight"),                           // f:12, l:12 | f:20, l:20
    ];

    let sstyle = Style::default().bg(Color::Blue).fg(Color::White);
    let dstyle = Style::default().bg(Color::LightYellow).fg(Color::White);

    let stupid = SeparatedList::new()
        .block(bounds)
        .defualt_style(dstyle)
        .selected_style(sstyle)
        .items(items)
        .item_display(ItemDisplay::Separated);

    f.render_stateful_widget(stupid, area, state);
}
