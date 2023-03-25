use ratatui::{style::Style, symbols::bar::HALF};

use super::DisplayLine;

/// Generates separator lines.
///
/// The separator line requires information about the previous line and the next line.
/// It uses the half block: "▄" to fill the line. The background is the same as the
/// background of the previous line, and the foreground is the background of the next
/// line. This tracks the style last displayed, and when a new line is genrated width
/// the next line's color, it "rotates" stored style so that the foreground (the bg Color
/// of the previously displayed item) is the background, and the new color (the backround of the
/// line which will be displayed after the separator) is set to foreground.
///
/// This also stores the default style of the list, to set the separators at the
/// very begining and very end to match the default style for the list widget.
#[derive(Clone, Copy)]
pub struct Separator {
    width: usize,
    default_style: Style,
    curr_style: Style,
}

impl<'a> Separator {
    pub(super) fn new(width: usize, style: Style) -> Self {
        //style.fg(Color::Reset);
        let default_style = match style.bg {
            Some(init_color) => Style::reset().bg(init_color).fg(init_color),
            None => Style::default(),
        };

        Separator {
            width,
            default_style,
            curr_style: default_style,
        }
    }

    pub(super) fn display_line(
        &mut self,
        must_display: bool,
        style: Option<Style>,
    ) -> DisplayLine<'a> {
        let style = style.unwrap_or(self.default_style);
        self.curr_style.bg = self.curr_style.fg;
        self.curr_style.fg = style.bg;

        DisplayLine {
            style: self.curr_style,
            line: gen_line(self.width).into(),
            must_display,
            left_indicator: HALF.into(),
            right_indicator: HALF.into(),
        }
    }
}

// avoid allocation of a String per Separator::display_line call.
lazy_static::lazy_static! {
    static ref HALF_LINE: &'static str = {
        let line = HALF.repeat(256);
        let mem = Box::from(line);
        Box::leak(mem)
    };
}

// Number of bytes in the HALF codepoint
const HALF_SIZE: usize = HALF.as_bytes().len();

#[inline]
fn gen_line(width: usize) -> &'static str {
    // split takes the byte offset, but must be on codepoint boundaries
    HALF_LINE.split_at(width * HALF_SIZE).0
}
