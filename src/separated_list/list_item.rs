use tui::{style::Style, text::Text};

/// An entry in a list. This is a direct copy of the [`tui::widgets::ListItem`].
///
/// It is remimplemented here because the provided list item keeps its members private,
/// and those values are needed in drawing the list. If tui-rs increased the visibility
/// scope for its `ListItem` this can be replaced.
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem<'a> {
    pub(super) content: Text<'a>,
    pub(super) style: Style,
}

impl<'a> ListItem<'a> {
    pub fn new<T>(content: T) -> ListItem<'a>
    where
        T: Into<Text<'a>>,
    {
        ListItem {
            content: content.into(),
            style: Style::default(),
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
