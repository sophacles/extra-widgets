//! A list widget with many display styling options.
//!
//! This list models its display on by rendering all the [`ListItem`] elements of `items` into
//! indivdual lines of text, and then moving a window over the lines to acheive the final view.
mod line_iters;
mod list_item;
mod list_state;
mod separator;
mod window_type;

use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::Spans,
    widgets::{Block, StatefulWidget, Widget},
};

pub use list_item::ListItem;
pub use list_state::ListState;
use separator::Separator;

/// A rendered line of text in the list widget. Multiple DisplayLines can be created from a single
/// [`ListItem`]. The window operates on an iterable of [`DiplayLine`]s
#[derive(Clone, Debug)]
struct DisplayLine<'a> {
    pub(super) style: Style,
    pub(super) line: Spans<'a>,
    pub(super) must_display: bool,
}

impl<'a> DisplayLine<'a> {
    /// Construct an empty DisplayLine (e.g as a placeholder)
    fn filler(x: &'static str) -> Self {
        Self {
            style: Style::default(),
            line: Spans::from(x),
            must_display: false,
        }
    }
}

/// Control how lines are rendered
pub enum ItemDisplay {
    /// Basic `ItemDisplay` simply renders each text line in the [`ListItem`] iterator into a
    /// display line.
    Basic,
    /// Separated `ItemDisplay` places a separator between each [`ListItem`] (including
    /// endcaps so items A, B, C will be rendered as `S A1 A2 S B1 S C1 S`)
    Separated,
}

/// Control how the window places itself with respect to the rendered lines, i.e. control the list
/// display of rendered lines.
pub enum WindowType {
    /// Diplay rendered lines so that the first selected [`ListItem`] is always visible. The location of
    /// the selected items within the display window is dependent on movement. This operates the
    /// way one naturally expects from a list widget - and "moves" the selection first, then the
    /// displayed lines if the selection otherwise wouldn't be displayed.
    SelectionScroll,
    /// Display the rendered lines so that the selected [`ListItem`] always displays in the same
    /// place on the screen. Effectively this always "moves the list" around the selection.
    Fixed,
}

impl WindowType {
    /// Iterate through the rendered display lines and produce the ones that should be shown in the
    /// window.
    fn get_display_lines<'a, I>(
        &self,
        items: I,
        window_size: usize,
        list_state: &mut ListState,
    ) -> impl Iterator<Item = DisplayLine<'a>>
    where
        I: Iterator<Item = DisplayLine<'a>>,
    {
        use WindowType::*;
        match self {
            SelectionScroll => window_type::selection_scroll(items, window_size, list_state),
            Fixed => window_type::fixed(items, window_size, list_state),
        }
    }
}

/// A general purpose List widget that has several modes of display
pub struct SeparatedList<'a> {
    block: Option<Block<'a>>,
    default_style: Style,
    selected_style: Style,
    window_type: WindowType,
    item_display: ItemDisplay,
    items: Vec<ListItem<'a>>,
}

impl<'a> SeparatedList<'a> {
    /// Wrap the list in a block (e.g. to set borders or a title).
    pub fn block(mut self, b: Block<'a>) -> Self {
        self.block = Some(b);
        self
    }

    /// The style that will be used for [`ListItem`]s that are not selected. If an item includes a
    /// style, that style will be patched into the default style.
    pub fn default_style(mut self, s: Style) -> Self {
        self.default_style = s;
        self
    }

    /// The style applied to lines of the selected item. If the this list uses [`ItemDisplay::Separated`]
    /// the surrounding separators will also be highlighted using this style.
    pub fn selected_style(mut self, s: Style) -> Self {
        self.selected_style = s;
        self
    }

    /// Set the [`ListItem`]s to be used for this list
    pub fn items(mut self, items: Vec<ListItem<'a>>) -> Self {
        self.items = items;
        self
    }

    /// Set the window type for this list
    pub fn window_type(mut self, wt: WindowType) -> Self {
        self.window_type = wt;
        self
    }

    /// Set the item display control
    pub fn item_display(mut self, it: ItemDisplay) -> Self {
        self.item_display = it;
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
            it.style = if i == selected {
                self.selected_style
            } else {
                self.default_style.patch(it.style)
            };

            line_iters::ToLines::new(it, i == selected)
        });

        let item_display: Box<dyn Iterator<Item = DisplayLine<'a>>> = match self.item_display {
            ItemDisplay::Basic => Box::new(line_iters::Basic::new(iter)),
            ItemDisplay::Separated => Box::new(line_iters::Separated::new(iter, sep)),
        };

        let lines = self
            .window_type
            .get_display_lines(item_display, area.height as usize, state);

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
        SeparatedList {
            items: vec![],
            block: None,
            default_style: Style::default(),
            selected_style: Style::default(),
            window_type: WindowType::SelectionScroll,
            item_display: ItemDisplay::Basic,
        }
    }
}
