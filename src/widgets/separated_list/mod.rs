mod line_iters;
mod list_item;
mod list_state;
pub(crate) mod separator;
mod viewport;

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Spans,
    widgets::{Block, StatefulWidget, Widget},
};

pub use list_item::ListItem;
pub use list_state::ListState;
pub use separator::Separator;

#[derive(Debug)]
struct DisplayLine<'a> {
    pub(super) style: Style,
    pub(super) line: Spans<'a>,
    pub(super) must_display: bool,
}

pub struct SeparatedList<'a> {
    block: Option<Block<'a>>,
    default_style: Style,
    selected_style: Style,
    items: Vec<ListItem<'a>>,
}

impl<'a> SeparatedList<'a> {
    pub fn new() -> Self {
        SeparatedList {
            items: vec![],
            block: None,
            default_style: Style::default(),
            selected_style: Style::default(),
        }
    }

    pub fn block(mut self, b: Block<'a>) -> Self {
        self.block = Some(b);
        self
    }

    pub fn defualt_style(mut self, s: Style) -> Self {
        self.default_style = s;
        self
    }

    pub fn selected_style(mut self, s: Style) -> Self {
        self.selected_style = s;
        self
    }

    pub fn items(mut self, items: Vec<ListItem<'a>>) -> Self {
        self.items = items;
        self
    }
}

impl<'a> StatefulWidget for SeparatedList<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        // Block is used for borders and such
        // Draw that first, and use the blank area inside the block for our own purposes
        let area = match self.block {
            None => area,
            Some(b) => {
                let inner = b.inner(area);
                b.render(area, buf);
                inner
            }
        };

        // set style for whole area
        buf.set_style(area, self.default_style);

        let sep = Separator::new(area.width as usize, self.default_style);

        let selected = state.selected;
        let iter = self.items.into_iter().enumerate().map(|(i, mut it)| {
            if i == selected {
                it.selected = true;
                it.style = self.selected_style;
            } else {
                it.style = self.default_style;
            }
            it
        });

        let lines = viewport::selection_scroll(
            line_iters::Separated::new(iter, sep),
            area.height as usize,
            state,
        );

        for (i, l) in lines.into_iter().enumerate() {
            let d_area = Rect {
                x: area.x,
                y: area.y + i as u16,
                height: 1,
                width: area.width,
            };
            buf.set_style(d_area, l.style);
            buf.set_spans(area.x, area.y + i as u16, &l.line, area.width);
        }
    }
}

impl<'a> Widget for SeparatedList<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = ListState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

impl<'a> Default for SeparatedList<'a> {
    fn default() -> Self {
        Self::new()
    }
}
