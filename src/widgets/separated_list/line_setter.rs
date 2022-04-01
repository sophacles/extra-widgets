use std::cell::RefCell;

use super::separator::Separator;
use super::{ListItem, ListState};

use tui::{layout::Rect, style::Style, text::Spans};

pub(super) struct LineSetter<'a> {
    area: Rect,
    items: Vec<ListItem<'a>>,
    raw_height: usize,
    state: RefCell<&'a mut ListState>,
    separate: bool,
    default_style: Style,
}

impl<'a> LineSetter<'a> {
    pub(super) fn new(
        area: Rect,
        items: Vec<ListItem<'a>>,
        state: &'a mut ListState,
        default_style: Style,
        selected_style: Style,
    ) -> Self {
        let mut raw_height: usize = 0;

        let items = items
            .into_iter()
            .enumerate()
            .scan(&mut raw_height, |h, (n, mut it)| {
                let height = **h;
                **h = height + it.height();
                it.line_pos = height;
                if n == state.selected {
                    it.selected = true;
                    it.style = selected_style;
                } else {
                    it.style = default_style;
                }
                Some(it)
            })
            .collect::<Vec<_>>();

        let state = RefCell::new(state);

        Self {
            area,
            items,
            state,
            raw_height,
            separate: false,
            default_style,
        }
    }

    pub(super) fn add_separators(mut self) -> Self {
        // add a space between each item
        let old_len = self.items.len();
        let mut new_items: Vec<ListItem<'a>> = Vec::with_capacity((self.items.len() * 2) + 1);
        let mut sep = Separator::new(self.area.width as usize, self.default_style);

        for (i, mut li) in self.items.into_iter().enumerate() {
            sep.cycle_color(li.style.bg);
            new_items.push(sep.get_list_item(li.line_pos + i));
            li.line_pos += i + 1;
            new_items.push(li);
        }

        sep.cycle_color(self.default_style.bg);
        new_items.push(sep.get_list_item(self.raw_height + old_len));

        self.raw_height += old_len + 1;
        self.separate = true;
        self.items = new_items;
        self
    }

    fn first_display_line(&self) -> u16 {
        // find it incase we put in separators ill take these microsecs back tho if
        // i can figure out the math
        let sel = self.items.iter().position(|it| it.selected).unwrap();
        let window_size: usize = self.area.height as usize;
        #[cfg(test)]
        println!("window_size: {}", window_size);

        #[cfg(test)]
        println!("ah: {}", self.area.height);

        // assume we will still be in the same window
        let mut window_first = self.state.borrow().old_window_first;
        let window_last = window_first + window_size.saturating_sub(1);

        // get selected item display
        let last_line_must_show: usize;
        let first_line_must_show: usize;
        if self.separate {
            // do the math to include the separator now
            last_line_must_show = (self.items[sel].last_line() + 1) as usize;
            first_line_must_show = self.items[sel].line_pos.saturating_sub(1);
        } else {
            last_line_must_show = self.items[sel].last_line() as usize;
            first_line_must_show = self.items[sel].line_pos;
        };

        #[cfg(test)]
        println!("last_line_must_show: {}", last_line_must_show);
        // make sure the selected item will be displayed, the selected item
        // must fit it's last line
        if last_line_must_show > window_last {
            #[cfg(test)]
            println!(
                "sub! last_line_must_show: {}, window_size: {}",
                last_line_must_show, window_size
            );
            window_first = last_line_must_show.saturating_sub(window_size - 1);
        }

        // it more importantly must fit it's first line, so:
        // do this second: if item is too big for window, display start
        if first_line_must_show < window_first {
            window_first = first_line_must_show;
        }

        self.state.borrow_mut().old_window_first = window_first;
        window_first as u16
    }

    pub(super) fn get_spans(&self) -> Vec<DisplayLine> {
        let mut spans: Vec<DisplayLine> = Vec::with_capacity(self.area.height as usize);

        let first = self.first_display_line() as usize;
        let mut current: usize = 0;

        #[cfg(test)]
        println!("first is: {}", first);
        'outer: for i in self.items.iter() {
            if i.last_line() < first {
                #[cfg(test)]
                println!("skipping because last line: {} < {}", i.last_line(), first);
                current += i.height();
                continue;
            }

            for s in i.content.lines.iter() {
                if current >= first {
                    let style = i.style;
                    let dl = DisplayLine {
                        style,
                        line: s.clone(),
                    };
                    spans.push(dl);
                }

                current += 1;
                if spans.len() >= self.area.height as usize {
                    //println!("finishing because spans is now: {} long", spans.len());
                    break 'outer;
                }
            }
        }
        spans
    }

    #[cfg(test)]
    fn print(&self) {
        for (i, it) in self.items.iter().enumerate() {
            println!(
                "{}) lp: {}, ll: {}, sel: {}",
                i,
                it.line_pos,
                it.last_line(),
                it.selected
            );
        }
    }
}

#[derive(Debug)]
pub(super) struct DisplayLine<'a> {
    pub(super) style: Style,
    pub(super) line: Spans<'a>,
}

#[cfg(test)]
mod test {

    use super::*;
    use tui::style::Color;

    // a common set of items to show. always work with this to make
    // reasoning about tests easier
    fn make_items<'a>() -> Vec<ListItem<'a>> {
        let items = vec![
            //                                     no sep     | sep
            ListItem::new("0:one"),               // f:0, l:0   | f:1, l:1
            ListItem::new("1:two\n.\n."),         // f:1, l:3   | f:3, l:5
            ListItem::new("2:three\n."),          // f:4, l:5   | f:7, l:8
            ListItem::new("3:four"),              // f:6, l:6   | f:10, l:10
            ListItem::new("4:five\n."),           // f:7, l:8   | f:12, l:13
            ListItem::new("5:six"),               // f:9, l:9   | f:15, l:15
            ListItem::new("6:seven\n...and 1/2"), // f:10, l:11 | f:17, l:18
            ListItem::new("7:eight"),             // f:12, l:12 | f:20, l:20
        ]; // last line overall = line on 12, height 13 | sep on 21, height = 22

        items
    }

    fn pspans<'a, I>(i: I)
    where
        I: Iterator<Item = &'a DisplayLine<'a>>,
    {
        for (i, s) in i.enumerate() {
            println!("spans{}:\n{:?}\n{:?}", i, s.style, s.line);
        }
    }

    #[derive(Debug)]
    // last_fully vis, offset line
    enum Move {
        Next(usize),
        Prev(usize),
    }

    #[test]
    fn test_first_display_line() {
        // for all these scenarios window is size 5
        let area = Rect {
            x: 1,
            y: 2,
            width: 8,
            height: 5,
        };
        let mut state = ListState::new(8);
        let cycle: Vec<u16> = vec![0, 0, 1, 2, 2, 1, 0];
        let moves: Vec<Move> = vec![
            Move::Next(0),
            Move::Next(0),
            Move::Next(1),
            Move::Prev(2),
            Move::Prev(2),
            Move::Prev(1),
            Move::Prev(0),
        ];
        for (idx, (i, m)) in cycle.into_iter().zip(moves).enumerate() {
            println!("State: {:?}", state);
            let tsetter = LineSetter::new(
                area,
                make_items(),
                &mut state,
                Style::default(),
                Style::default(),
            );
            let l = tsetter.first_display_line();
            assert_eq!(l, i, "(at {}) expect: {} {:?} (got: {})", idx, i, m, l);

            drop(tsetter);

            match m {
                Move::Next(a) => {
                    state.old_window_first = a;
                    state.cycle_next();
                }
                Move::Prev(a) => {
                    state.old_window_first = a;
                    state.cycle_prev();
                }
            }
        }
    }

    #[test]
    fn test_first_display_line_2() {
        let area = Rect {
            x: 1,
            y: 2,
            width: 8,
            height: 5,
        };
        let mut state = ListState::new(8);
        state.old_window_first = 4;
        state.select(5);
        let items = make_items();
        let tsetter = LineSetter::new(area, items, &mut state, Style::default(), Style::default());

        assert_eq!(tsetter.first_display_line(), 5);
        assert_eq!(state.old_window_first, 5);
    }

    #[test]
    fn test_first_display_line_3() {
        let area = Rect {
            x: 1,
            y: 2,
            width: 8,
            height: 5,
        };
        let mut state = ListState::new(8);
        state.old_window_first = 7;
        state.select(7);
        let items = make_items();
        let mut tsetter =
            LineSetter::new(area, items, &mut state, Style::default(), Style::default());

        assert_eq!(tsetter.first_display_line(), 8);
        assert_eq!(state.old_window_first, 8);

        let items = make_items();
        let mut tsetter =
            LineSetter::new(area, items, &mut state, Style::default(), Style::default())
                .add_separators();

        assert_eq!(tsetter.first_display_line(), 17);
        assert_eq!(state.old_window_first, 17);
    }

    #[test]
    fn test_constructor() {
        let mut state = ListState::new(8);
        let items = make_items();
        let tsetter = LineSetter::new(
            Rect::default(),
            items,
            &mut state,
            Style::default(),
            Style::default(),
        );

        assert_eq!(tsetter.raw_height, 13);
        assert_eq!(tsetter.items[0].last_line(), 0);
        assert_eq!(tsetter.items[1].last_line(), 3);
        assert_eq!(tsetter.items[2].last_line(), 5);
        assert_eq!(tsetter.items[3].last_line(), 6);
        assert_eq!(tsetter.items[4].last_line(), 8);
        assert_eq!(tsetter.items[5].last_line(), 9);
        assert_eq!(tsetter.items[6].last_line(), 11);
        assert_eq!(tsetter.items[7].last_line(), 12);
    }

    #[test]
    fn test_sepmath() {
        let area = Rect {
            x: 1,
            y: 2,
            width: 8,
            height: 5,
        };
        let mut state = ListState::new(8);
        let items = make_items();
        let mut tsetter =
            LineSetter::new(area, items, &mut state, Style::default(), Style::default());

        let old_rh = tsetter.raw_height;
        tsetter = tsetter.add_separators();

        assert_ne!(tsetter.raw_height, old_rh);
        assert_eq!(tsetter.raw_height, 22);
        assert_eq!(tsetter.items[0].last_line(), 0);
        assert_eq!(tsetter.items[1].last_line(), 1);
        assert_eq!(tsetter.items[2].last_line(), 2);
        assert_eq!(tsetter.items[3].last_line(), 5);
        assert_eq!(tsetter.items[4].last_line(), 6);
        assert_eq!(tsetter.items[5].last_line(), 8);
        assert_eq!(tsetter.items[6].last_line(), 9);
        assert_eq!(tsetter.items[7].last_line(), 10);
        assert_eq!(tsetter.items[8].last_line(), 11);
        assert_eq!(tsetter.items[9].last_line(), 13);
        assert_eq!(tsetter.items[10].last_line(), 14);
        assert_eq!(tsetter.items[11].last_line(), 15);
        assert_eq!(tsetter.items[12].last_line(), 16);
        assert_eq!(tsetter.items[13].last_line(), 18);
        assert_eq!(tsetter.items[14].last_line(), 19);
        assert_eq!(tsetter.items[15].last_line(), 20);
        assert_eq!(tsetter.items[16].last_line(), 21);
    }

    #[test]
    fn test_spans() {
        let mut state = ListState::new(8);
        let items = make_items();
        let area = Rect {
            x: 1,
            y: 2,
            width: 8,
            height: 5,
        };
        let bgstyle = Style::default().bg(Color::Blue);
        let tsetter = LineSetter::new(area, items, &mut state, Style::default(), bgstyle);
        let spans = tsetter.get_spans();
        assert_eq!(spans.len(), 5);

        state.old_window_first = 2;
        state.selected = 4;

        let items = make_items();
        let tsetter = LineSetter::new(area, items, &mut state, Style::default(), bgstyle);
        let spans = tsetter.get_spans();
        assert_eq!(spans.len(), 5);

        state.old_window_first = 5;
        state.selected = 5;

        let items = make_items();
        let tsetter = LineSetter::new(area, items, &mut state, Style::default(), bgstyle);
        tsetter.print();
        let spans = tsetter.get_spans();
        pspans(spans.iter());
        assert_eq!(spans.len(), 5);
        assert!(spans[4].style.bg.is_some());
    }

    #[test]
    fn test_spans_sep() {
        let mut state = ListState::new(8);
        state.old_window_first = 3;
        state.select(7);
        let items = make_items();
        let area = Rect {
            x: 1,
            y: 2,
            width: 8,
            height: 17,
        };
        let bgstyle = Style::default().bg(Color::Blue);
        let tsetter =
            LineSetter::new(area, items, &mut state, Style::default(), bgstyle).add_separators();
        let spans = tsetter.get_spans();

        assert_eq!(spans.len(), 17);
        assert!(spans[14].style.fg.is_some());
        assert!(spans[15].style.bg.is_some());
        assert!(spans[16].style.bg.is_some());
    }
}
