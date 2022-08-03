use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    Frame,
};

use extra_widgets::separated_list::{ItemDisplay, SeparatedList, WindowType};

use super::super::{words, AppState};

pub fn fixed<B: Backend>(area: Rect, state: &mut AppState, f: &mut Frame<B>) {
    let demo_items = words();
    let demo_list = SeparatedList::default()
        .default_style(Style::reset().bg(Color::Black).fg(Color::White))
        .selected_style(Style::default().bg(Color::Blue).fg(Color::White))
        .items(demo_items)
        .item_display(ItemDisplay::Basic)
        .window_type(WindowType::Fixed);

    f.render_stateful_widget(demo_list, area, &mut state.examples);
}
