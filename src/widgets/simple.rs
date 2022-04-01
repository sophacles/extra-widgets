use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Widget},
};

pub struct Simple<'a> {
    msg: String,
    block: Option<Block<'a>>,
    style: Style,
}

impl<'a> Simple<'a> {
    pub fn new() -> Self {
        Simple {
            msg: String::new(),
            block: None,
            style: Style::default(),
        }
    }

    pub fn block(mut self, b: Block<'a>) -> Self {
        self.block = Some(b);
        self
    }

    pub fn msg(mut self, msg: &str) -> Self {
        self.msg = String::from(msg);
        self
    }

    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }
}

impl<'a> Widget for Simple<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
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

        let x = area.width / 2;
        let y = area.height / 2;
        buf.set_string(x, y, self.msg, self.style);
    }
}

impl<'a> Default for Simple<'a> {
    fn default() -> Self {
        Self::new()
    }
}
