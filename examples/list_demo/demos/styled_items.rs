use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    Frame,
};

use extra_widgets::styled_list::{Indicator, ItemDisplay, LineIndicators, StyledList};

use super::super::{words, AppState};

pub fn styled_items<B: Backend>(area: Rect, state: &mut AppState, f: &mut Frame<B>) {
    let demo_items = words();
    let orange = Style::default().bg(Color::Rgb(242, 147, 5));
    let demo_items =
        demo_items
            .into_iter()
            .enumerate()
            .map(|(i, it)| if i % 3 == 0 { it.style(orange) } else { it });

    let demo_list = StyledList::new(demo_items)
        .default_style(Style::reset().bg(Color::Black).fg(Color::White))
        .selected_style(Style::default().add_modifier(Modifier::BOLD))
        .selected_indicator(
            LineIndicators::default()
                .set_right(Indicator::Char("|"))
                .set_left(Indicator::Char("|")),
        )
        .show_left_indicator()
        .show_right_indicator()
        .item_display(ItemDisplay::Separated);
    f.render_stateful_widget(demo_list, area, &mut state.examples);
}
