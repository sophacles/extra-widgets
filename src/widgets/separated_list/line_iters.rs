use std::collections::VecDeque;

use tui::{style::Style, text::Spans};

use super::{DisplayLine, ListItem, Separator};

pub enum ItemDisplay {
    Basic,
    Separated,
}

struct ToLines<'a> {
    style: Style,
    text_items: VecDeque<Spans<'a>>,
    selected: bool,
}

impl<'a> From<ListItem<'a>> for ToLines<'a> {
    fn from(item: ListItem<'a>) -> Self {
        Self {
            style: item.style,
            text_items: VecDeque::from(item.content.lines),
            selected: item.selected,
        }
    }
}

impl<'a> Iterator for ToLines<'a> {
    type Item = DisplayLine<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let res = DisplayLine {
            style: self.style,
            line: self.text_items.pop_front()?,
            must_display: self.selected,
        };
        Some(res)
    }
}

pub(super) struct Basic<'a, I>
where
    I: IntoIterator<Item = ListItem<'a>>,
{
    items: I::IntoIter,
    current: Option<ToLines<'a>>,
}

impl<'a, I> Basic<'a, I>
where
    I: IntoIterator<Item = ListItem<'a>>,
{
    pub(super) fn new(items: I) -> Self {
        let mut items = items.into_iter();
        let current = items.next().map(|it| it.into());
        Self { items, current }
    }
}

impl<'a, I> Iterator for Basic<'a, I>
where
    I: IntoIterator<Item = ListItem<'a>>,
{
    type Item = DisplayLine<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let lines = self.current.as_mut()?;
        match lines.next() {
            Some(l) => Some(l),
            None => {
                let mut new_lines: ToLines = self.items.next()?.into();
                let res = new_lines.next();
                self.current = Some(new_lines);
                res
            }
        }
    }
}

pub(super) struct Separated<'a, I>
where
    I: IntoIterator<Item = ListItem<'a>>,
{
    items: std::iter::Peekable<I::IntoIter>,
    current: Option<ToLines<'a>>,
    separator: Separator,
    last_line_selected: bool,
}

impl<'a, I> Separated<'a, I>
where
    I: IntoIterator<Item = ListItem<'a>>,
{
    pub(super) fn new(items: I, separator: Separator) -> Self {
        let mut items = items.into_iter().peekable();
        // kick start the iterator to just handle the "current ended, must add separator"
        // case immediately so we start with a separator.
        let current = items.peek().map(|next| {
            //separator.cycle_color(next.style.bg);
            ToLines {
                style: Style::default(),
                text_items: VecDeque::new(),
                selected: next.selected,
            }
        });
        Self {
            items,
            current,
            separator,
            last_line_selected: false,
        }
    }
}

impl<'a, I> Iterator for Separated<'a, I>
where
    I: IntoIterator<Item = ListItem<'a>>,
{
    type Item = DisplayLine<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let lines = self.current.as_mut()?;

        let res = match lines.next() {
            Some(l) => l,
            None => match self.items.next() {
                Some(next) => {
                    let next_selected = next.selected;
                    self.separator.cycle_color(next.style.bg);
                    self.current = Some(next.into());
                    self.separator
                        .display_line(next_selected || self.last_line_selected)
                }
                None => {
                    self.current = None;
                    self.separator.cycle_default();
                    self.separator.display_line(self.last_line_selected)
                }
            },
        };
        self.last_line_selected = res.must_display;
        Some(res)
    }
}

#[cfg(test)]
mod test {
    use tui::style::Color;

    use super::*;
    use tui::symbols::bar::HALF;

    #[test]
    fn to_lines() {
        let style = Style::default().fg(Color::Red).bg(Color::Blue);
        let it = ListItem::new("a\nb\nc").style(style);

        for (dl, s) in ToLines::from(it).zip(["a", "b", "c"]) {
            assert_eq!(dl.line, Spans::from(s));
            assert_eq!(dl.style, style);
        }
    }

    #[test]
    fn to_lines_selected() {
        let mut item = ListItem::new("a\nb");
        item.selected = true;

        for i in ToLines::from(item) {
            assert!(i.must_display)
        }
    }

    #[test]
    fn basic_display_lines() {
        let mut items = vec![ListItem::new("a\nb\nc"), ListItem::new("d\ne")];
        items[1].selected = true;
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
        let mut items = vec![
            ListItem::new("a\nb\nc"),
            ListItem::new("d\ne").style(sstyle),
        ];
        items[1].selected = true;
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
            ListItem::new("a\nb\nc").style(sstyle),
            ListItem::new("d\ne"),
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
        let mut items = vec![
            ListItem::new("a\nb\nc"),
            ListItem::new("d\ne").style(sstyle),
            ListItem::new("f\ng"),
        ];
        items[1].selected = true;
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
            ListItem::new("a\nb\nc").style(fstyle),
            ListItem::new("d\ne").style(sstyle),
            ListItem::new("f\ng").style(lstyle),
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
