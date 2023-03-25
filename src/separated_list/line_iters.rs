use std::iter::Enumerate;

use tui::{style::Style, text::Spans};

use super::{DisplayLine, LineIndicators, ListItem, Separator};

/// A struct for iterating through display lines given an item and a selection state
pub(super) struct ToLines<'a> {
    style: Style,
    text_items: Enumerate<std::vec::IntoIter<Spans<'a>>>,
    //text_items: VecDeque<(usize, usize, Spans<'a>)>,
    indicators: LineIndicators,
    selected: bool,
    line_count: usize,
}

impl<'a> ToLines<'a> {
    pub(super) fn new(item: ListItem<'a>, selected: bool) -> Self {
        let line_count = item.height();
        let text_items = item.content.lines.into_iter().enumerate();

        //let text_items = VecDeque::from_iter(text_items);
        Self {
            style: item.style,
            text_items,
            indicators: item.indicators,
            selected,
            line_count,
        }
    }

    pub(super) fn empty_with_selection(selected: bool) -> Self {
        Self {
            style: Style::default(),
            text_items: Vec::new().into_iter().enumerate(),
            selected,
            indicators: LineIndicators::default(),
            line_count: 0,
        }
    }
}

impl<'a> Iterator for ToLines<'a> {
    type Item = DisplayLine<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let (i, line) = self.text_items.next()?;
        let res = DisplayLine {
            style: self.style,
            line,
            must_display: self.selected,
            left_indicator: self.indicators.left.fill_char(i, self.line_count).into(),
            right_indicator: self.indicators.right.fill_char(i, self.line_count).into(),
        };
        Some(res)
    }
}

/// Basic line iterator, will render each display line it encounters
pub(super) struct Basic<'a, I>
where
    I: IntoIterator<Item = ToLines<'a>>,
{
    items: std::iter::Flatten<I::IntoIter>,
}

impl<'a, I> Basic<'a, I>
where
    I: IntoIterator<Item = ToLines<'a>>,
{
    pub(super) fn new(items: I) -> Self {
        let items = items.into_iter().flatten();
        Self { items }
    }
}

impl<'a, I> Iterator for Basic<'a, I>
where
    I: IntoIterator<Item = ToLines<'a>>,
{
    type Item = DisplayLine<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        self.items.next()
    }
}

/// Separated line iterator - will place a Separator between each ListItem as it renders
/// DisplayLines
pub(super) struct Separated<'a, I>
where
    I: IntoIterator<Item = ToLines<'a>>,
{
    tolines_iter: std::iter::Peekable<I::IntoIter>,
    lines_iter: Option<ToLines<'a>>,
    separator: Separator,
    prev_was_selected: bool,
}

impl<'a, I> Separated<'a, I>
where
    I: IntoIterator<Item = ToLines<'a>>,
{
    pub(super) fn new(items: I, separator: Separator) -> Self {
        let mut tolines_iter = items.into_iter().peekable();
        // kick start the iterator to just handle the "lines_iter ended, must add separator"
        // case immediately so we start with a separator.
        let lines_iter = tolines_iter
            .peek()
            .map(|next| ToLines::empty_with_selection(next.selected));
        Self {
            tolines_iter,
            lines_iter,
            separator,
            prev_was_selected: false,
        }
    }
}

impl<'a, I> Iterator for Separated<'a, I>
where
    I: IntoIterator<Item = ToLines<'a>>,
{
    type Item = DisplayLine<'a>;
    // This is basically a flatten and interleave.
    //
    // self.lines_iter is the ToLines that is currently being turned into DisplayLines.
    // The separator tracks styles and generates the correct displayline
    fn next(&mut self) -> Option<Self::Item> {
        // if lines_iter has been set to None, iterator is over
        let lines = self.lines_iter.as_mut()?;

        // otherwise get the next line from the iterator
        let res = match lines.next() {
            Some(l) => l,
            None => match self.tolines_iter.next() {
                Some(next) => {
                    // if either or the neighbors are lines from the selected item,
                    // the separator line is marked "must_display" also.
                    let must_display = next.selected || self.prev_was_selected;
                    let next_style = next.style;
                    self.lines_iter = Some(next);
                    self.separator.display_line(must_display, Some(next_style))
                }
                None => {
                    self.lines_iter = None;
                    self.separator.display_line(self.prev_was_selected, None)
                }
            },
        };
        self.prev_was_selected = res.must_display;
        Some(res)
    }
}

#[cfg(test)]
mod test {
    use tui::{style::Color, symbols::bar::HALF};

    use super::*;

    #[test]
    fn to_lines() {
        let style = Style::default().fg(Color::Red).bg(Color::Blue);
        let it = ListItem::new("a\nb\nc").style(style);

        for (dl, s) in ToLines::new(it, false).zip(["a", "b", "c"]) {
            assert_eq!(dl.line, Spans::from(s));
            assert_eq!(dl.style, style);
        }
    }

    #[test]
    fn to_lines_selected() {
        let item = ListItem::new("a\nb");

        for i in ToLines::new(item, true) {
            assert!(i.must_display)
        }
    }

    #[test]
    fn basic_display_lines() {
        let items = vec![
            ToLines::new(ListItem::new("a\nb\nc"), false),
            ToLines::new(ListItem::new("d\ne"), true),
        ];
        for (dl, (t, s)) in Basic::new(items).zip([
            ("a", false),
            ("b", false),
            ("c", false),
            ("d", true),
            ("e", true),
        ]) {
            assert_eq!(dl.line, Spans::from(t));
            assert_eq!(dl.must_display, s);
        }
    }

    #[test]
    fn separated_display_lines_end_selected() {
        let sstyle = Style::default().bg(Color::Red).fg(Color::Blue);
        let items = vec![
            ToLines::new(ListItem::new("a\nb\nc"), false),
            ToLines::new(ListItem::new("d\ne").style(sstyle), true),
        ];
        for (dl, (t, s, bg, fg)) in
            Separated::new(items, Separator::new(1, Style::default())).zip([
                (HALF, false, None, None),
                ("a", false, None, None),
                ("b", false, None, None),
                ("c", false, None, None),
                (HALF, true, None, Some(Color::Red)),
                ("d", true, Some(Color::Red), Some(Color::Blue)),
                ("e", true, Some(Color::Red), Some(Color::Blue)),
                (HALF, true, Some(Color::Red), None),
            ])
        {
            assert_eq!(dl.line, Spans::from(t));
            assert_eq!(dl.must_display, s);
            assert_eq!(dl.style.bg, bg);
            assert_eq!(dl.style.fg, fg);
        }
    }

    #[test]
    fn separated_display_lines_begin_selected() {
        let sstyle = Style::default().bg(Color::Red).fg(Color::Blue);
        let mut items = vec![
            ToLines::new(ListItem::new("a\nb\nc").style(sstyle), true),
            ToLines::new(ListItem::new("d\ne"), false),
        ];
        items[0].selected = true;
        for (dl, (t, s, bg, fg)) in
            Separated::new(items, Separator::new(1, Style::default())).zip([
                (HALF, true, None, Some(Color::Red)),
                ("a", true, Some(Color::Red), Some(Color::Blue)),
                ("b", true, Some(Color::Red), Some(Color::Blue)),
                ("c", true, Some(Color::Red), Some(Color::Blue)),
                (HALF, true, Some(Color::Red), None),
                ("d", false, None, None),
                ("e", false, None, None),
                (HALF, false, None, None),
            ])
        {
            assert_eq!(dl.line, Spans::from(t));
            assert_eq!(dl.must_display, s);
            assert_eq!(dl.style.bg, bg);
            assert_eq!(dl.style.fg, fg);
        }
    }

    #[test]
    fn separated_display_lines_middle_selected() {
        let sstyle = Style::default().bg(Color::Red).fg(Color::Blue);
        let items = vec![
            ToLines::new(ListItem::new("a\nb\nc"), false),
            ToLines::new(ListItem::new("d\ne").style(sstyle), true),
            ToLines::new(ListItem::new("f\ng"), false),
        ];
        for (dl, (t, s, bg, fg)) in
            Separated::new(items, Separator::new(1, Style::default())).zip([
                (HALF, false, None, None),
                ("a", false, None, None),
                ("b", false, None, None),
                ("c", false, None, None),
                (HALF, true, None, Some(Color::Red)),
                ("d", true, Some(Color::Red), Some(Color::Blue)),
                ("e", true, Some(Color::Red), Some(Color::Blue)),
                (HALF, true, Some(Color::Red), None),
                ("f", false, None, None),
                ("g", false, None, None),
                (HALF, false, None, None),
            ])
        {
            assert_eq!(dl.line, Spans::from(t));
            assert_eq!(dl.must_display, s, "line: {:?}", dl);
            assert_eq!(dl.style.bg, bg);
            assert_eq!(dl.style.fg, fg);
        }
    }

    #[test]
    fn separated_display_lines_middle_selected_styled_items() {
        let fstyle = Style::default().bg(Color::Cyan);
        let sstyle = Style::default().bg(Color::Red).fg(Color::Blue);
        let lstyle = Style::default().bg(Color::Green);
        let mut items = vec![
            ToLines::new(ListItem::new("a\nb\nc").style(fstyle), false),
            ToLines::new(ListItem::new("d\ne").style(sstyle), true),
            ToLines::new(ListItem::new("f\ng").style(lstyle), false),
        ];
        items[1].selected = true;
        for (dl, (t, s, bg, fg)) in
            Separated::new(items, Separator::new(1, Style::default())).zip([
                (HALF, false, None, Some(Color::Cyan)),
                ("a", false, Some(Color::Cyan), None),
                ("b", false, Some(Color::Cyan), None),
                ("c", false, Some(Color::Cyan), None),
                (HALF, true, Some(Color::Cyan), Some(Color::Red)),
                ("d", true, Some(Color::Red), Some(Color::Blue)),
                ("e", true, Some(Color::Red), Some(Color::Blue)),
                (HALF, true, Some(Color::Red), Some(Color::Green)),
                ("f", false, Some(Color::Green), None),
                ("g", false, Some(Color::Green), None),
                (HALF, false, Some(Color::Green), None),
            ])
        {
            assert_eq!(dl.line, Spans::from(t));
            assert_eq!(dl.must_display, s, "line: {:?}", dl);
            assert_eq!(dl.style.bg, bg);
            assert_eq!(dl.style.fg, fg);
        }
    }
}
