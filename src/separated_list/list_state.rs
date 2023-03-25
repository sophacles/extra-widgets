use std::cmp::min;

#[cfg(feature = "serde")]
use serde_derive::{Deserialize, Serialize};

/// State for a [`StyledList`](super::StyledList)
///
/// This state tracks the selected item in a list, and provides methods for cycling the list.
/// The size of the list is the number of [`ListItem`](super::ListItem)s to cycle through.
///
/// panics if created or resized to have a size of 0
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ListState {
    pub(super) size: usize,
    pub(super) selected: usize,
    pub(super) window_first: usize,
}

impl ListState {
    /// Create a new state for a list of length `size`.
    pub fn new(size: usize) -> Self {
        let mut res = ListState {
            size: 1,
            selected: 0,
            window_first: 0,
        };
        res.resize(size);
        res
    }

    /// Set the position of the first DisplayLine of the selection.
    pub(super) fn set_pos(&mut self, pos: usize) {
        self.window_first = pos;
    }

    /// Select the next [ListItem](super::ListItem) without wrapping
    pub fn next(&mut self) {
        self.selected = min(self.selected + 1, self.size - 1)
    }

    /// Select the previous [ListItem](super::ListItem) without wrapping
    pub fn prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Select the next item in the list. If the current item is the last [ListItem`(super::ListItem), it will
    /// move the selection to the first [ListItem](super::ListItem)
    pub fn cycle_next(&mut self) {
        self.selected = (self.selected + 1) % self.size;
    }

    /// Select the previous item in the list. If the current item is the first [ListItem](super::ListItem), it will
    /// move the selection to the last [ListItem](super::ListItem)
    pub fn cycle_prev(&mut self) {
        self.selected = (self.selected + self.size - 1) % self.size;
    }

    /// Specify which [ListItem](super::ListItem) is selected. If the selection is beyond the end of the list, the
    /// last item will be selected.
    pub fn select(&mut self, n: usize) {
        self.selected = n;
        if self.selected >= self.size {
            self.selected = self.size.saturating_sub(1);
        }
    }

    /// Get the index of the selected [ListItem](super::ListItem)
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// set the number of [ListItems](super::ListItem) in the list.
    pub fn resize(&mut self, size: usize) {
        if size == 0 {
            panic!("ListState has invalid size: 0");
        }
        self.size = size;
        if self.selected >= self.size {
            self.selected = self.size.saturating_sub(1);
        }
    }
}

impl Default for ListState {
    fn default() -> Self {
        ListState::new(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycles() {
        let mut s = ListState::new(3);
        assert_eq!(s.selected(), 0);
        s.cycle_next();
        assert_eq!(s.selected(), 1);
        s.cycle_next();
        assert_eq!(s.selected(), 2);
        s.cycle_next();
        assert_eq!(s.selected(), 0);

        s.cycle_prev();
        assert_eq!(s.selected(), 2);
        s.cycle_prev();
        assert_eq!(s.selected(), 1);
        s.cycle_prev();
        assert_eq!(s.selected(), 0);
    }

    #[test]
    fn movement() {
        let mut s = ListState::new(3);
        assert_eq!(s.selected(), 0);
        s.prev();
        assert_eq!(s.selected(), 0);
        s.next();
        assert_eq!(s.selected(), 1);
        s.next();
        assert_eq!(s.selected(), 2);
        s.next();
        assert_eq!(s.selected(), 2);
    }

    #[test]
    fn resize() {
        let mut s = ListState::new(3);
        s.select(2);
        assert_eq!(s.selected(), 2);
        s.resize(2);
        assert_eq!(s.selected(), 1);
        s.resize(4);
        assert_eq!(s.selected(), 1);
        s.cycle_prev();
        s.cycle_prev();
        assert_eq!(s.selected(), 3);
    }

    #[test]
    #[should_panic]
    fn zero_size_create() {
        let _ = ListState::new(0);
    }

    #[test]
    #[should_panic]
    fn zero_size_resize() {
        let mut s = ListState::new(1);
        s.resize(0);
    }
}
