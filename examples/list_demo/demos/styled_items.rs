use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    Frame,
};

use extra_widgets::separated_list::{ItemDisplay, SeparatedList};

use super::super::{words, AppState};

pub fn styled_items<B: Backend>(area: Rect, state: &mut AppState, f: &mut Frame<B>) {
    let demo_items = words();
    let orange = Style::default().bg(Color::Rgb(242, 147, 5));
    let demo_items = demo_items
        .into_iter()
        .enumerate()
        .map(|(i, it)| if i % 3 == 0 { it.style(orange) } else { it })
        .collect();

    let demo_list = SeparatedList::default()
        .default_style(Style::reset().bg(Color::Black).fg(Color::White))
        .selected_style(Style::default().bg(Color::Blue).fg(Color::White))
        .items(demo_items)
        .item_display(ItemDisplay::Separated);
    f.render_stateful_widget(demo_list, area, &mut state.examples);
}
