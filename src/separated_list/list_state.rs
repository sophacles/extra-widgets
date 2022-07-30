/// State for a [`SeparatedList`](super::SeparatedList)
///
/// This state tracks the selected item in a list, and provides methods for cycling the list.
/// The size of the list is the number of [`ListItem`](super::ListItem)s to cycle through.
#[derive(Debug, Default)]
pub struct ListState {
    pub(super) size: usize,
    pub(super) selected: usize,
    pub(super) window_first: usize,
}

impl ListState {
    /// Create a new state for a list of length `size`.
    pub fn new(size: usize) -> Self {
        let mut res = Self::default();
        res.resize(size);
        res
    }

    /// Set the position of the first DisplayLine of the selection.
    pub(super) fn set_pos(&mut self, pos: usize) {
        self.window_first = pos;
    }

    /// Select the next item in the list. If the current item is the last [`ListItem`](super::ListItem), it will
    /// move the selection to the first [`ListItem`](super::ListItem)
    pub fn cycle_next(&mut self) {
        self.selected = (self.selected + 1) % self.size;
    }

    /// Select the previous item in the list. If the current item is the first [`ListItem`](super::ListItem), it will
    /// move the selection to the last [`ListItem`](super::ListItem)
    pub fn cycle_prev(&mut self) {
        self.selected = (self.selected + self.size - 1) % self.size;
    }

    /// Specify which [`ListItem`](super::ListItem) is selected. If the selection is beyond the end of the list, the
    /// last item will be selected.
    pub fn select(&mut self, n: usize) {
        self.selected = n;
        if self.selected >= self.size {
            self.selected = self.size.saturating_sub(1);
        }
    }

    /// Get the index of the selected [`ListItem`](super::ListItem)
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// set the number of [`ListItems`](super::ListItem) in the list.
    pub fn resize(&mut self, size: usize) {
        self.size = size;
        if self.selected >= self.size {
            self.selected = self.size.saturating_sub(1);
        }
    }
}
