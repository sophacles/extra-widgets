#[derive(Debug, Default)]
pub struct ListState {
    pub(super) size: usize,
    pub(super) selected: usize,
    pub(super) window_first: usize,
}

impl ListState {
    pub fn new(size: usize) -> Self {
        let mut res = Self::default();
        res.resize(size);
        res
    }

    pub fn set_pos(&mut self, pos: usize) {
        self.window_first = pos;
    }

    pub fn cycle_next(&mut self) {
        self.selected = (self.selected + 1) % self.size;
    }

    pub fn cycle_prev(&mut self) {
        self.selected = (self.selected + self.size - 1) % self.size;
    }

    pub fn select(&mut self, n: usize) {
        self.selected = n;
        if self.selected >= self.size {
            self.selected = self.size.saturating_sub(1);
        }
    }

    pub fn resize(&mut self, size: usize) {
        self.size = size;
        if self.selected >= self.size {
            self.selected = self.size.saturating_sub(1);
        }
    }
}
