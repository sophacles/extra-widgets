use tui::{style::Style, text::Text};

#[derive(Debug, Clone, PartialEq)]
pub struct ListItem<'a> {
    pub(super) content: Text<'a>,
    pub(super) style: Style,
    pub(super) line_pos: usize,
    pub(super) selected: bool,
}

impl<'a> ListItem<'a> {
    pub fn new<T>(content: T) -> ListItem<'a>
    where
        T: Into<Text<'a>>,
    {
        ListItem {
            content: content.into(),
            style: Style::default(),
            line_pos: 0,
            selected: false,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn height(&self) -> usize {
        self.content.height()
    }
}
