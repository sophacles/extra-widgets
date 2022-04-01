pub mod separated_list;
pub mod simple;

pub use separated_list::{ListItem, ListState, SeparatedList};
pub use simple::Simple;

use tui::layout::Rect;

// helpers i'll figure out later
pub trait RectHelpers {
    fn vert_window(&self, y_start: u16, size: u16, from: &Rect) -> Rect;
}

impl RectHelpers for Rect {
    fn vert_window(&self, y_start: u16, size: u16, from: &Rect) -> Rect {
        Rect {
            y: y_start,
            height: size,
            x: from.x,
            width: from.width,
        }
    }
}
