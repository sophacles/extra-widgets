use std::{error::Error, io};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};

use extra_widgets::separated_list::{ItemDisplay, ListItem, ListState, SeparatedList};

mod demos;

static WORDS: &str = include_str!("../wordlist.txt");

fn words<'a>() -> Vec<ListItem<'a>> {
    WORDS.trim_end().split('\n').map(ListItem::new).collect()
}

#[derive(Debug)]
enum Focus {
    Picker,
    Example,
}

impl Focus {
    fn toggle(&mut self) {
        use Focus::*;
        match self {
            Picker => *self = Example,
            Example => *self = Picker,
        }
    }
}

#[derive(Debug)]
pub struct AppState {
    focus: Focus,
    picker: ListState,
    examples: ListState,
}

impl AppState {
    fn new(n_picker: usize, n_examples: usize) -> Self {
        Self {
            focus: Focus::Picker,
            picker: ListState::new(n_picker),
            examples: ListState::new(n_examples),
        }
    }

    fn switch_focus(&mut self) {
        self.focus.toggle();
    }

    fn move_up(&mut self) {
        match self.focus {
            Focus::Picker => self.picker.cycle_prev(),
            Focus::Example => self.examples.cycle_prev(),
        }
    }

    fn move_down(&mut self) {
        match self.focus {
            Focus::Picker => self.picker.cycle_next(),
            Focus::Example => self.examples.cycle_next(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = AppState::new(1, words().len());

    loop {
        let mstate = &mut state;
        let _ = terminal.draw(|f| draw(mstate, f));

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) if c == 'j' => {
                    state.move_down();
                }
                KeyCode::Char(c) if c == 'k' => {
                    state.move_up();
                }
                KeyCode::Char(c) if c == 'h' || c == 'l' => {
                    state.switch_focus();
                }
                KeyCode::Char(_) => {
                    break;
                }
                _ => {}
            };
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn draw<B: Backend>(state: &mut AppState, f: &mut Frame<B>) {
    let app_area = f.size();
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Min(1),
                Constraint::Min(1),
                Constraint::Percentage(100),
            ]
            .as_ref(),
        );

    let chunks = layout.split(app_area);
    //println!("{:?}", chunks);
    let bar_area = chunks[0];
    // this could be done other ways, i just like how it looks
    let border_area = chunks[1];
    let main_area = chunks[2];

    // draw top bar
    let top_text = Spans::from(vec![
        Span::styled("Controls:", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" 'h' - "),
        Span::styled("left,", Style::default().add_modifier(Modifier::ITALIC)),
        Span::raw(" 'j' - "),
        Span::styled("down,", Style::default().add_modifier(Modifier::ITALIC)),
        Span::raw(" 'k' - "),
        Span::styled("up,", Style::default().add_modifier(Modifier::ITALIC)),
        Span::raw(" 'l' - "),
        Span::styled("right", Style::default().add_modifier(Modifier::ITALIC)),
    ]);
    let top_text = Paragraph::new(top_text).alignment(Alignment::Center);
    f.render_widget(top_text, bar_area);

    let top_border = Block::default().borders(Borders::TOP);
    f.render_widget(top_border, border_area);

    let list_layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints(
            [
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(50),
            ]
            .as_ref(),
        );
    let chunks = list_layout.split(main_area);

    let (select_list_area, demo_frame_area, code_area) = (chunks[0], chunks[1], chunks[2]);

    let demo_frame = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::DarkGray))
        .style(Style::default())
        .title("demo")
        .title_alignment(Alignment::Center);

    let demo_list_area = demo_frame.inner(demo_frame_area);
    f.render_widget(demo_frame, demo_frame_area);
    let selections = vec![
        ListItem::new("basic"), // 0
        ListItem::new("separated"),
        ListItem::new("fixed"),
        ListItem::new("styled items"),
    ];
    state.picker.resize(4);

    let demo_list_area = demo_list_area.inner(&Margin {
        vertical: 2,
        horizontal: 2,
    });

    let code = match state.picker.selected() {
        0 => {
            demos::basic(demo_list_area, state, f);
            include_str!("demos/basic.rs")
        }
        1 => {
            demos::separated(demo_list_area, state, f);
            include_str!("demos/separated.rs")
        }
        2 => {
            demos::fixed(demo_list_area, state, f);
            include_str!("demos/fixed.rs")
        }
        3 => {
            demos::styled_items(demo_list_area, state, f);
            include_str!("demos/styled_items.rs")
        }
        _ => unreachable!(),
    };

    let bstyle = Style::default().fg(Color::White);
    let select_bounds = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title("select demo")
        .border_type(BorderType::Thick)
        .style(bstyle);

    let select_list = SeparatedList::new(selections)
        .block(select_bounds)
        .default_style(Style::reset().bg(Color::Black).fg(Color::White))
        .selected_style(Style::default().bg(Color::Blue).fg(Color::White))
        .item_display(ItemDisplay::Separated);
    f.render_stateful_widget(select_list, select_list_area, &mut state.picker);

    let code_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title("code")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Plain)
        .style(bstyle);
    let code = Paragraph::new(code).block(code_block);
    f.render_widget(code, code_area);
}
