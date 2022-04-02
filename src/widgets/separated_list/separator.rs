use tui::{
    style::{Color, Style},
    symbols::bar::HALF,
};

use super::{line_iters::DisplayLine, ListItem};

#[derive(Clone, Copy)]
pub struct Separator {
    width: usize,
    init_style: Style,
    curr_style: Style,
}

impl<'a> Separator {
    pub(super) fn new(width: usize, style: Style) -> Self {
        let init_style = if style.bg.is_some() {
            Style::default().bg(style.bg.unwrap())
        } else {
            Style::default()
        };

        Separator {
            width,
            init_style,
            curr_style: init_style,
        }
    }

    pub(super) fn cycle_color(&mut self, c: Option<Color>) {
        self.curr_style.bg = self.curr_style.fg;
        self.curr_style.fg = c;
    }

    pub(super) fn cycle_default(&mut self) {
        self.curr_style.bg = self.curr_style.fg;
        self.curr_style.fg = self.init_style.bg;
    }

    pub(super) fn get_list_item(&self, pos: usize) -> ListItem<'a> {
        let line = HALF.repeat(self.width);
        let mut res = ListItem::new(line);
        res = res.style(self.curr_style);
        res.line_pos = pos;
        res
    }
}
