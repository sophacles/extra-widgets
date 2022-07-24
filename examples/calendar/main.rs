use std::{error::Error, io, rc::Rc};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};

use time::{macros::date, Date, Month, OffsetDateTime};

use widgets::calendar::{Calendar, CalendarEventStore, DateStyler};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        let _ = terminal.draw(|f| draw(f));

        if let Event::Key(key) = event::read()? {
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
    let app_area = f.size();

    let calarea = Rect {
        x: app_area.x + 1,
        y: app_area.y + 1,
        height: app_area.height - 1,
        width: app_area.width - 1,
    };

    let mut start = OffsetDateTime::now_local()
        .unwrap()
        .date()
        .replace_month(Month::January)
        .unwrap()
        .replace_day(1)
        .unwrap();

    let list = make_list();

    for chunk in split_rows(calarea)
        .into_iter()
        .map(|row| split_cols(row))
        .flatten()
    {
        let cal = cals::get_cal(start.month(), start.year(), &list);
        f.render_widget(cal, chunk);
        start = start.replace_month(start.month().next()).unwrap();
    }
}

fn split_rows(area: Rect) -> Vec<Rect> {
    let list_layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(
            [
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(33),
            ]
            .as_ref(),
        );

    list_layout.split(area)
}

fn split_cols(area: Rect) -> Vec<Rect> {
    let list_layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints(
            [
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        );

    list_layout.split(area)
}

fn make_list() -> CalendarEventStore {
    let hs = Style::default()
        .fg(Color::Red)
        .add_modifier(Modifier::UNDERLINED);

    let mut list = CalendarEventStore::today(
        Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::Blue),
    );

    list.add(date!(2022 - 12 - 25), hs);
    list.add(date!(2022 - 07 - 4), hs);
    list
}

mod cals {
    use super::*;

    pub(super) fn get_cal<'a, S: DateStyler>(m: Month, y: i32, es: S) -> Calendar<'a, S> {
        use Month::*;
        match m {
            July => july(m, y, es),
            _ => default(m, y, es),
        }
    }

    fn default<'a, S: DateStyler>(m: Month, y: i32, es: S) -> Calendar<'a, S> {
        let default_style = Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::Rgb(50, 50, 50));

        Calendar::new(Date::from_calendar_date(y, m, 1).unwrap(), es)
            .show_month(Style::default())
            .default_style(default_style)
    }

    fn july<'a, S: DateStyler>(m: Month, y: i32, es: S) -> Calendar<'a, S> {
        let header_style = Style::default()
            .add_modifier(Modifier::BOLD)
            .fg(Color::Green);

        let default_style = Style::default()
            .add_modifier(Modifier::BOLD)
            .bg(Color::Rgb(50, 50, 50));

        Calendar::new(OffsetDateTime::now_local().unwrap().date(), es)
            .show_surrounding(Style::default().add_modifier(Modifier::DIM))
            .show_weekdays(header_style)
            .default_style(default_style)
            .show_month(Style::default())
    }
}
