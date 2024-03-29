use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    Frame,
};

use extra_widgets::styled_list::{ItemDisplay, StyledList};

use super::super::{words, AppState};

pub fn separated<B: Backend>(area: Rect, state: &mut AppState, f: &mut Frame<B>) {
    let demo_items = words();
    let demo_list = StyledList::new(demo_items)
        .default_style(Style::reset().bg(Color::Black).fg(Color::White))
        .selected_style(Style::default().bg(Color::Blue).fg(Color::White))
        .item_display(ItemDisplay::Separated);
    f.render_stateful_widget(demo_list, area, &mut state.examples);
}
