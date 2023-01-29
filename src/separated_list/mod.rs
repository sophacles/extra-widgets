//! A list widget with many display styling options.
//!
//! This list models its display on by rendering all the [`ListItem`] elements of `items` into
//! indivdual lines of text, and then moving a window over the lines to acheive the final view.
// This whole thing is implemented as pipeline of iterators applying a series of transforms
// on the iterator of ListItems the SeparatedList is generic on.
//
// The transforms go like this:
//
// ListItem -> ToLines -> DisplayLine
//
// ToLines is an internal iterator over the text of of the ListItem's line of text, turing them
// into DisplayLines
//
// DisplayLine can be turned into a fully styled entry displayed in the SeparatedList widget.
//
// The rendering pipeline goes like this:
//
// items
// -> Apply selection styling based on state
// -> flatten iter of ToLines to iter of DisplayLine, insert any DisplayLines for the ItemDisplay
//    style
// -> Filter the DiplayLines iterator to only the lines to draw via the window iterator.
//
// The window iterators process in a single pass so this pipeline is at worst O(n) (althougth if
// window fills up before finishing the display, it will stop iteration before that.
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

pub use list_item::{Indicator, LineIndicators, ListItem};
pub use list_state::ListState;
use separator::Separator;

/// A rendered line of text in the list widget. Multiple DisplayLines can be created from a single
/// [`ListItem`]. The window operates on an iterable of [`DiplayLine`]s
#[derive(Clone, Debug)]
struct DisplayLine<'a> {
    pub(super) style: Style,
    pub(super) line: Spans<'a>,
    pub(super) must_display: bool,
    pub(super) left_indicator: Spans<'a>,
    pub(super) right_indicator: Spans<'a>,
}

/// Control how lines are rendered
#[derive(Debug, Copy, Clone)]
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
    Fixed(usize),
}

/// A general purpose List widget that has several modes of display
pub struct SeparatedList<'a, I>
where
    I: IntoIterator<Item = ListItem<'a>>,
{
    block: Option<Block<'a>>,
    default_style: Style,
    selected_style: Style,
    selected_indicator: LineIndicators,
    show_left_indicator: bool,
    show_right_indicator: bool,
    window_type: WindowType,
    item_display: ItemDisplay,
    items: I,
}

impl<'a, I> SeparatedList<'a, I>
where
    I: IntoIterator<Item = ListItem<'a>>,
{
    pub fn new(items: I) -> Self {
        Self {
            items,
            block: None,
            default_style: Style::default(),
            selected_style: Style::default(),
            selected_indicator: LineIndicators::default(),
            show_left_indicator: false,
            show_right_indicator: false,
            window_type: WindowType::SelectionScroll,
            item_display: ItemDisplay::Basic,
        }
    }
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

    /// The indicators to use for the selected item
    pub fn selected_indicator(mut self, indicator: LineIndicators) -> Self {
        self.selected_indicator = indicator;
        self
    }

    /// Display the left indicator column - if not set the left indicator will not be displayed
    pub fn show_left_indicator(mut self) -> Self {
        self.show_left_indicator = true;
        self
    }

    /// Display the right indicator column - if not set the right indicator will not be displayed
    pub fn show_right_indicator(mut self) -> Self {
        self.show_right_indicator = true;
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

impl<'a, I> StatefulWidget for SeparatedList<'a, I>
where
    I: IntoIterator<Item = ListItem<'a>>,
{
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

        // Start the pipeline: appy indicators and patch in appropriate stylings.
        // Then convert to a ToLines.
        let selected = state.selected;
        let iter = self.items.into_iter().enumerate().map(|(i, mut it)| {
            if i == selected {
                it = it.indicators(self.selected_indicator);
                it.style = self
                    .default_style
                    .patch(it.style.patch(self.selected_style));
            } else {
                it.style = self.default_style.patch(it.style);
            }

            line_iters::ToLines::new(it, i == selected)
        });

        // Next step of pipeline, apply DisplayLine renderer
        let item_display = self.item_display.display_iter(iter, sep);

        // Filter the lines to those in the current view window
        let lines = self
            .window_type
            .get_display_lines(item_display, area.height as usize, state);

        // Draw the lines into the window.
        for (i, l) in lines.into_iter().enumerate() {
            let y = area.y + i as u16;
            // first fill the whole line area
            let d_area = Rect {
                x: area.x,
                y,
                height: 1,
                width: area.width,
            };
            buf.set_style(d_area, l.style);

            let mut x = area.x;
            let mut line_width = area.width;

            // show the left indicator and adjust the display area for the item text
            if self.show_left_indicator {
                buf.set_spans(x, y, &l.left_indicator, 1);
                x += 1;
                line_width -= 1;
            }

            // show the right indicator and adjust the display area for the item text
            if self.show_right_indicator {
                buf.set_spans(x + line_width - 1, y, &l.right_indicator, 1);
                line_width -= 1;
            }

            // show the item text
            buf.set_spans(x, y, &l.line, line_width);
        }
    }
}

impl<'a, I> Widget for SeparatedList<'a, I>
where
    I: IntoIterator<Item = ListItem<'a>>,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = ListState::default();
        StatefulWidget::render(self, area, buf, &mut state);
    }
}

// Private impls of conveience functions in render
impl<'a> DisplayLine<'a> {
    /// Construct an empty DisplayLine (e.g as a placeholder)
    fn filler(x: &'static str) -> Self {
        Self {
            style: Style::default(),
            line: Spans::from(x),
            must_display: false,
            left_indicator: Spans::from(x),
            right_indicator: Spans::from(x),
        }
    }
}

impl ItemDisplay {
    fn display_iter<'a, I>(self, iter: I, sep: Separator) -> DisplayIter<'a, I>
    where
        I: IntoIterator<Item = line_iters::ToLines<'a>>,
    {
        match self {
            ItemDisplay::Basic => DisplayIter::Basic(line_iters::Basic::new(iter)),
            ItemDisplay::Separated => DisplayIter::Separated(line_iters::Separated::new(iter, sep)),
        }
    }
}

enum DisplayIter<'a, I>
where
    I: IntoIterator<Item = line_iters::ToLines<'a>>,
{
    Basic(line_iters::Basic<'a, I>),
    Separated(line_iters::Separated<'a, I>),
}

impl<'a, I> Iterator for DisplayIter<'a, I>
where
    I: IntoIterator<Item = line_iters::ToLines<'a>>,
{
    type Item = DisplayLine<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DisplayIter::Basic(ref mut b) => b.next(),
            DisplayIter::Separated(ref mut s) => s.next(),
        }
    }
}

impl WindowType {
    /// Iterate through the rendered display lines and produce the ones that should be shown in the
    /// window.
    fn get_display_lines<'a, I>(
        self,
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
            Fixed(at) => window_type::fixed(items, at, window_size, list_state),
        }
    }
}
