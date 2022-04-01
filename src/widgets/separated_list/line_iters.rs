use crate::widgets::ListItem;

struct ViewPort {
    pos: usize,
    height: usize,
}

impl ViewPort {
    pub fn top(&self) -> usize {
        self.pos
    }

    pub fn bottom(&self) -> usize {
        self.pos + self.height.saturating_sub(1)
    }

    pub fn move_to_hold(&mut self, item: &DisplayItem) {
        // If the last line to show is below the window, move the window down to it
        if item.last_display_line() > self.bottom() {
            self.pos = item.last_display_line().saturating_sub(self.height - 1);
        }

        // At this point, we either didn't change position above, which means that the
        // window may have been too low because the selection moved up sine the last
        // run.
        // Or we adjusted to fit the item, but it's larger than the viewport.
        //
        // In either case move the window so that the start of the item is displayed
        if item.first_display_line() < self.pos {
            self.pos = item.first_display_line();
        }
    }
}

struct DisplayItem<'a> {
    item: ListItem<'a>,
    line_pos: usize,
    separated: bool,
}

impl<'a> DisplayItem<'a> {
    fn first_display_line(&self) -> usize {
        if self.separated {
            self.line_pos.saturating_sub(1)
        } else {
            self.line_pos
        }
    }

    pub fn height(&self) -> usize {
        self.item.height()
    }

    pub(super) fn last_display_line(&self) -> usize {
        if self.separated {
            self.height()
        } else {
            self.height() - 1
        }
    }
}
