use tui::{
    style::{Color, Style},
    symbols::bar::HALF,
};

use super::DisplayLine;

#[derive(Clone, Copy)]
pub struct Separator {
    width: usize,
    init_style: Style,
    curr_style: Style,
}

impl<'a> Separator {
    pub(super) fn new(width: usize, style: Style) -> Self {
        //style.fg(Color::Reset);
        let init_style = match style.bg {
            Some(init_color) => Style::reset().bg(init_color).fg(init_color),
            None => Style::default(),
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

    pub(super) fn display_line(&self, must_display: bool) -> DisplayLine<'a> {
        let line = HALF.repeat(self.width);
        DisplayLine {
            style: self.curr_style,
            line: line.into(),
            must_display,
        }
    }
}
