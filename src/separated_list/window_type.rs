use std::fmt::Display;

use bounded_vec_deque::BoundedVecDeque;

use super::{DisplayLine, ListState};

/// A small state machine to track the display of selected items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelState {
    NotSeen,
    Started(usize),
    Complete,
}

impl SelState {
    fn toggle(&mut self, sel: bool, index: usize) {
        use SelState::*;
        *self = match (*self, sel) {
            (NotSeen, true) => Started(index),
            (Started(_), false) => Complete,
            _ => *self,
        };
    }
}

impl Default for SelState {
    fn default() -> Self {
        SelState::NotSeen
    }
}

/// Tracking for the display window used in selection scroll. The display window is the slice of
/// lines that should be rendered to that screen. `top` is the first element of that slice.
///
/// When possible, the items displayed on the screen should remain the same, even if the selection
/// changes. This is stored in the `goal` member. The window may be advanced past the goal to ensure
/// the entire selection is displayed.
///
/// Restricting the window to an index prevents the top from going any further than that index.
/// This is set to the first line of the selection, so that the selection can be displayed in full.
struct Window {
    goal: usize,
    top: usize,
    restriction: Option<usize>,
}

impl Window {
    fn new(prev_pos: usize) -> Self {
        Self {
            goal: prev_pos,
            top: 0,
            restriction: None,
        }
    }

    /// Idempotent method to restrict the winow the first time it's called with
    /// SelState::Started(idx), which will set the restriction to idx.
    fn restrict(&mut self, state: SelState) {
        if self.restriction.is_none() {
            if let SelState::Started(i) = state {
                self.restriction = Some(i);
            }
        }
    }

    /// move the top of the window forward.
    fn advance(&mut self) {
        self.top += 1;
    }

    /// Has the top reached or passed the goal?
    fn is_aligned(&self) -> bool {
        self.top >= self.goal
    }

    /// Has the top reached the restriction?
    fn is_restricted(&self) -> bool {
        match self.restriction {
            Some(s) => self.top >= s,
            None => false,
        }
    }
}

impl Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "goal: {}, curr: {}, restriction: {:?}",
            self.goal, self.top, self.restriction
        )
    }
}

/// Line selector for [`WindowType::SelectionScroll`](super::WindowType::SelectionScroll).
pub(super) fn selection_scroll<'a, I>(
    items: I,
    window_size: usize,
    list_state: &mut ListState,
) -> <BoundedVecDeque<I::Item> as IntoIterator>::IntoIter
where
    I: IntoIterator<Item = DisplayLine<'a>>,
{
    let mut window = Window::new(list_state.window_first);
    let mut sel_state = SelState::NotSeen;

    // This stores the lines that will be displayed.
    let mut buffer = BoundedVecDeque::<I::Item>::new(window_size);

    for (i, l) in items.into_iter().enumerate() {
        sel_state.toggle(l.must_display, i);
        window.restrict(sel_state);
        // Fill the window before advancing it.
        if !buffer.is_full() {
            buffer.push_back(l);
            continue;
        }

        // since the buffer is full, check in on the state machine
        match sel_state {
            // if we haven't seen selection yet, push the window forward
            SelState::NotSeen => {
                window.advance();
                buffer.push_back(l);
            }

            // as long as the window isn't restricted, advance so to fit the whole selection. This
            // catches the cases where seletion moved "up" beyond the first line previously
            // displayed.
            SelState::Started(_) => {
                if window.is_restricted() {
                    break;
                } else {
                    window.advance();
                    buffer.push_back(l);
                }
            }
            // Since the whole selection is on screen, the quit either on alignment or restriction.
            // This catches the cases where the selection moved "down" to include lines off the
            // screen, and where the selected items has more lines than the current window.
            SelState::Complete => {
                if window.is_aligned() || window.is_restricted() {
                    break;
                } else {
                    window.advance();
                    buffer.push_back(l);
                }
            }
        }
    }

    list_state.set_pos(window.top);
    buffer.into_iter()
}

/// line selector for [`WindowType::Fixed`](super::WindowType::Fixed).
pub(super) fn fixed<'a, I>(
    items: I,
    at: usize,
    window_size: usize,
    _list_state: &mut ListState,
) -> <BoundedVecDeque<I::Item> as IntoIterator>::IntoIter
where
    I: IntoIterator<Item = DisplayLine<'a>>,
{
    let mut sel_state = SelState::default();
    let mut buffer =
        //BoundedVecDeque::from_iter(std::iter::repeat(DisplayLine::filler("")).take(4), 4);
        BoundedVecDeque::from_iter(std::iter::repeat(DisplayLine::filler("")).take(at), at);

    for (i, dl) in items.into_iter().enumerate() {
        sel_state.toggle(dl.must_display, i);
        // always try to fill the window
        match sel_state {
            SelState::NotSeen => {
                buffer.push_back(dl);
            }
            _ => {
                buffer.set_max_len(window_size);
                buffer.push_back(dl);
                if buffer.is_full() {
                    break;
                }
            }
        }
    }
    buffer.into_iter()
}

#[cfg(test)]
mod test {
    use super::*;
    use tui::style::Style;
    use tui::text::Spans;

    #[test]
    fn sel_state_toggle() {
        use SelState::*;
        let mut state = SelState::default();
        for (i, (val, exp_state)) in [
            (false, NotSeen),
            (true, Started(1)),
            (true, Started(1)),
            (false, Complete),
            (false, Complete),
        ]
        .into_iter()
        .enumerate()
        {
            state.toggle(val, i);
            assert_eq!(state, exp_state);
        }
    }

    fn make_list<'a>(sel_start: usize, sel_end: usize) -> impl Iterator<Item = DisplayLine<'a>> {
        let l = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];

        l.into_iter().enumerate().map(move |(i, s)| {
            let must_display = i >= sel_start && i <= sel_end;
            DisplayLine {
                style: Style::default(),
                line: Spans::from(s),
                must_display,
                left_indicator: " ".into(),
                right_indicator: " ".into(),
            }
        })
    }

    #[test]
    fn starts_fitting() {
        // starts: |a B c| d e f g h i j
        // result: a B c
        let mut state = ListState::new(10);
        state.set_pos(0);
        let res: Vec<DisplayLine> = selection_scroll(make_list(1, 1), 3, &mut state).collect();

        assert_eq!(res[0].line.0[0].content, "a");
        assert_eq!(res[1].line.0[0].content, "b");
        assert_eq!(res[2].line.0[0].content, "c");

        assert!(!res[0].must_display);
        assert!(res[1].must_display);
        assert!(!res[2].must_display);
    }

    #[test]
    fn fits_end() {
        // starts: |a b C| d e f g h i j
        // result: a b C
        let mut state = ListState::new(10);
        state.set_pos(0);
        let res: Vec<DisplayLine> = selection_scroll(make_list(2, 2), 3, &mut state).collect();

        assert_eq!(res[0].line.0[0].content, "a");
        assert_eq!(res[1].line.0[0].content, "b");
        assert_eq!(res[2].line.0[0].content, "c");

        assert!(!res[0].must_display);
        assert!(!res[1].must_display);
        assert!(res[2].must_display);
    }

    #[test]
    fn slides_to_selection() {
        // starts: |a b c| D E f g h i j
        // result: c D E
        let mut state = ListState::new(10);
        state.set_pos(0);
        let res: Vec<DisplayLine> = selection_scroll(make_list(3, 4), 3, &mut state).collect();

        assert_eq!(res[0].line.0[0].content, "c");
        assert_eq!(res[1].line.0[0].content, "d");
        assert_eq!(res[2].line.0[0].content, "e");

        assert!(!res[0].must_display);
        assert!(res[1].must_display);
        assert!(res[2].must_display);
    }

    #[test]
    fn stops_at_fixed() {
        // starts: a b c D E |f g h| i j
        // result: D E f
        let mut state = ListState::new(10);
        state.set_pos(5);
        let res: Vec<DisplayLine> = selection_scroll(make_list(3, 4), 3, &mut state).collect();

        assert_eq!(res[0].line.0[0].content, "d");
        assert_eq!(res[1].line.0[0].content, "e");
        assert_eq!(res[2].line.0[0].content, "f");

        assert!(res[0].must_display);
        assert!(res[1].must_display);
        assert!(!res[2].must_display);
    }

    #[test]
    fn stops_at_fixed_sel_too_big() {
        // starts: a b c D E |F G h| i j
        // result: D E F
        let mut state = ListState::new(10);
        state.set_pos(5);
        let res: Vec<DisplayLine> = selection_scroll(make_list(3, 6), 3, &mut state).collect();

        assert_eq!(res[0].line.0[0].content, "d");
        assert_eq!(res[1].line.0[0].content, "e");
        assert_eq!(res[2].line.0[0].content, "f");

        assert!(res[0].must_display);
        assert!(res[1].must_display);
        assert!(res[2].must_display);
    }

    #[test]
    fn stops_at_sliding_sel_too_big() {
        // starts: |a b c| D E F G h i j
        // result: D E F
        let mut state = ListState::new(10);
        state.set_pos(0);
        let res: Vec<DisplayLine> = selection_scroll(make_list(3, 6), 3, &mut state).collect();

        assert_eq!(res[0].line.0[0].content, "d");
        assert_eq!(res[1].line.0[0].content, "e");
        assert_eq!(res[2].line.0[0].content, "f");

        assert!(res[0].must_display);
        assert!(res[1].must_display);
        assert!(res[2].must_display);
    }
}
