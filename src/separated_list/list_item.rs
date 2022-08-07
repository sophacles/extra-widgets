use tui::{style::Style, text::Text};

/// An Item in the list
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem<'a> {
    pub(super) content: Text<'a>,
    pub(super) style: Style,
    pub(super) indicators: LineIndicators,
}

impl<'a> ListItem<'a> {
    pub fn new<T>(content: T) -> ListItem<'a>
    where
        T: Into<Text<'a>>,
    {
        ListItem {
            content: content.into(),
            style: Style::default(),
            indicators: LineIndicators::default(),
        }
    }

    /// Set the style for this item. This style will be patched into the default style, and will
    /// have selected style patched into it.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// How many rows this item will take on display
    pub fn height(&self) -> usize {
        self.content.height()
    }

    /// set the indicators for this item. These will be replaced with the lists's
    /// selected_indicator if it has been set and the item is selected.
    pub fn indicators(mut self, indicators: LineIndicators) -> Self {
        self.indicators = indicators;
        self
    }
}

/// Container for holding the [Indicator]s for the left and right indicator columns
#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct LineIndicators {
    pub(super) left: Indicator,
    pub(super) right: Indicator,
}

impl LineIndicators {
    pub fn set_left(mut self, left: Indicator) -> Self {
        self.left = left;
        self
    }

    pub fn set_right(mut self, right: Indicator) -> Self {
        self.right = right;
        self
    }
}

/// An indicator for an item.
///
/// Each indicator is a single column wide, and used to decorate a [ListItem] that is displayed.
/// Since [ListItems] may be multiple lines, various strategies are available for how to display
/// the indicator - see the variants for details
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Indicator {
    /// each line of text for the item will display this char in the indicator column
    Char(&'static str),
    /// the last line of text for the item will display this char in the indicator column
    LastLine(&'static str),
    /// the first line of text for the item will display this char in the indicator column
    FirstLine(&'static str),
    /// the idx line of text for the item will display this char in the indicator column. If the
    /// idx is greater than the number of lines to be displayed, the last line of text will display
    /// the indicator char.
    IdxOrLast(usize, &'static str),
}

impl Indicator {
    /// Get the indicator char for the line. The `lines` parameter is used to determine last line.
    pub(crate) fn fill_char(&self, line_idx: usize, lines: usize) -> &'static str {
        use Indicator::*;
        match *self {
            Char(c) => c,
            FirstLine(c) => {
                if line_idx == 0 {
                    c
                } else {
                    " "
                }
            }
            LastLine(c) => {
                if line_idx == lines - 1 {
                    c
                } else {
                    " "
                }
            }
            IdxOrLast(target, c) => {
                if line_idx == target || std::cmp::min(target, lines - 1) == line_idx {
                    c
                } else {
                    " "
                }
            }
        }
    }
}

impl Default for Indicator {
    fn default() -> Self {
        Indicator::Char(" ")
    }
}
