use std::fmt::Display;

use bounded_vec_deque::BoundedVecDeque;

use super::{DisplayLine, ListState};

pub enum WindowType {
    SelectionScroll,
    Fixed,
}

impl WindowType {
    pub(super) fn get_display_lines<'a, I>(
        &self,
        items: I,
        window_size: usize,
        list_state: &mut ListState,
    ) -> impl Iterator<Item = DisplayLine<'a>>
    where
        I: Iterator<Item = DisplayLine<'a>>,
    {
        use WindowType::*;
        match self {
            SelectionScroll => selection_scroll(items, window_size, list_state),
            Fixed => fixed(items, window_size, list_state),
        }
    }
}

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

struct Window {
    goal_index: usize,
    curr_index: usize,
    fixed: Option<usize>,
}

impl Window {
    fn new(prev_pos: usize) -> Self {
        Self {
            goal_index: prev_pos,
            curr_index: 0,
            fixed: None,
        }
    }

    fn fix(&mut self, state: SelState) {
        if self.fixed.is_none() {
            if let SelState::Started(i) = state {
                self.fixed = Some(i);
            }
        }
    }

    fn advance(&mut self) {
        if self.goal_index == self.curr_index {
            self.goal_index += 1;
        }
        self.curr_index += 1;
    }

    fn is_aligned(&self) -> bool {
        self.goal_index == self.curr_index
    }

    fn can_advance(&self) -> bool {
        if let Some(s) = self.fixed {
            self.curr_index < s
        } else {
            true
        }
    }
}

impl Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "goal: {}, curr: {}, fixed: {:?}",
            self.goal_index, self.curr_index, self.fixed
        )
    }
}

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

    let mut buffer = BoundedVecDeque::<I::Item>::new(window_size);

    // if we haven't hit the end condition before hitting the end of the input iter,
    // just fall off and return whatever is buffered
    for (i, l) in items.into_iter().enumerate() {
        sel_state.toggle(l.must_display, i);
        window.fix(sel_state);
        // always try to fill the window
        if !buffer.is_full() {
            buffer.push_back(l);
            continue;
        }

        match sel_state {
            // if we haven't seen selection yet, push the window forward
            SelState::NotSeen => {
                window.advance();
                buffer.push_back(l);
            }

            SelState::Started(_) => {
                if window.can_advance() {
                    window.advance();
                    buffer.push_back(l);
                } else {
                    break;
                }
            }
            SelState::Complete => {
                // It kind of looks like the first and second break can be consolidated.
                // looks can be deceiving:
                // The first break is the happy path break, we want to break here because the
                // window is aligned with the target, and the selection is fully shown.
                // The second break is the "give up" path, because the window can't be advanced
                // in search of alignment, but isn't aligned either.

                if window.is_aligned() {
                    break;
                } else if window.can_advance() {
                    window.advance();
                    buffer.push_back(l);
                } else {
                    break;
                }
            }
        }
    }

    list_state.set_pos(window.curr_index);
    buffer.into_iter()
}

pub(super) fn fixed<'a, I>(
    items: I,
    window_size: usize,
    _list_state: &mut ListState,
) -> <BoundedVecDeque<I::Item> as IntoIterator>::IntoIter
where
    I: IntoIterator<Item = DisplayLine<'a>>,
{
    let mut sel_state = SelState::default();
    let mut buffer =
        BoundedVecDeque::from_iter(std::iter::repeat(DisplayLine::filler("")).take(4), 4);

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

        assert_eq!(res[0].must_display, false);
        assert_eq!(res[1].must_display, true);
        assert_eq!(res[2].must_display, false);
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

        assert_eq!(res[0].must_display, false);
        assert_eq!(res[1].must_display, false);
        assert_eq!(res[2].must_display, true);
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

        assert_eq!(res[0].must_display, false);
        assert_eq!(res[1].must_display, true);
        assert_eq!(res[2].must_display, true);
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

        assert_eq!(res[0].must_display, true);
        assert_eq!(res[1].must_display, true);
        assert_eq!(res[2].must_display, false);
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

        assert_eq!(res[0].must_display, true);
        assert_eq!(res[1].must_display, true);
        assert_eq!(res[2].must_display, true);
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

        assert_eq!(res[0].must_display, true);
        assert_eq!(res[1].must_display, true);
        assert_eq!(res[2].must_display, true);
    }
}
