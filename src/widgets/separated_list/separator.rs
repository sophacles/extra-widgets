use tui::{
    style::{Color, Style},
    symbols::bar::HALF,
};

use super::ListItem;

#[derive(Clone, Copy)]
pub struct Separator {
    width: usize,
    style: Style,
}

impl<'a> Separator {
    pub(super) fn new(width: usize, style: Style) -> Self {
        let style = if style.bg.is_some() {
            Style::default().bg(style.bg.unwrap())
        } else {
            Style::default()
        };

        Separator { width, style }
    }

    pub(super) fn cycle_color(&mut self, c: Option<Color>) {
        self.style.bg = self.style.fg;
        self.style.fg = c;
    }

    pub(super) fn get_list_item(&self, pos: usize) -> ListItem<'a> {
        let line = HALF.repeat(self.width);
        let mut res = ListItem::new(line);
        res.style(self.style);
        res.line_pos = pos;
        res
    }
}
