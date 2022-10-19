use std::{error::Error, io};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::Color,
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use extra_widgets::{bg, bold, fg, italic, line, text, underlined};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        let _ = terminal.draw(|f| draw(f));

        if let Event::Key(key) = event::read()? {
            #[allow(clippy::single_match)]
            match key.code {
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

fn draw<B: Backend>(f: &mut Frame<B>) {
    let result = text! {
        underlined!("Some fancy text for you!");
        "";
        line!("Hi, I'm ", bold!("BOLD"));
        line!("Hi, I'm ", italic!(bold!("BOLD and Italic")));
        line!(
            fg!("r", Color::Red),
            fg!("a", Color::Rgb(255, 165, 0)),
            fg!("i", Color::LightYellow),
            fg!("n", Color::LightGreen),
            fg!("b", Color::Cyan),
            fg!("o", Color::Blue),
            fg!("w", Color::Magenta),
            " ",
            bg!(fg!("text", Color::Black), Color::White),
        );
    };

    let code = r#"
text! {
    underlined!("Some fancy text for you!");
    "";
    line!("Hi, I'm ", bold!("BOLD"));
    line!("Hi, I'm ", italic!(bold!("BOLD and Italic")));
    line!(fg!("blue", Color::Blue));
    line!(
        fg!("r", Color::Red),
        fg!("a", Color::Rgb(255, 165, 0)),
        fg!("i", Color::LightYellow),
        fg!("n", Color::LightGreen),
        fg!("b", Color::Cyan),
        fg!("o", Color::Blue),
        fg!("w", Color::Magenta),
        " ",
        bg!(fg!("text", Color::Black), Color::White),
    );
};"#;

    let (code_pane, result_pane) = {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(f.size());
        (chunks[0], chunks[1])
    };

    make_block("code", code, code_pane, f);
    make_block("result", result, result_pane, f);
}

fn make_block<'a, B: Backend>(title: &str, text: impl Into<Text<'a>>, pos: Rect, f: &mut Frame<B>) {
    let b = Block::default().title(title).borders(Borders::all());
    let inner = b.inner(pos);
    f.render_widget(b, pos);

    let middle = get_middle(inner);
    let p = Paragraph::new(text);
    f.render_widget(p, middle);
}

fn get_middle(area: Rect) -> Rect {
    area.inner(&Margin {
        vertical: 2,
        horizontal: 5,
    })
}
