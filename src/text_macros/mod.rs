//! macros for building and styling text for tui.

/// styles text into a span with the bold modifier set. The argument must evaluate to something
/// that implements [`Into<Span>`](tui::text::Span)
#[macro_export]
macro_rules! bold {
    ($e:expr) => {{
        let mut s = ::tui::text::Span::from($e);
        s.style = s.style.add_modifier(::tui::style::Modifier::BOLD);
        s
    }};
}

/// styles text into a span with the italic modifier set. The argument must evaluate to something
/// that implements [`Into<Span>`](tui::text::Span)
#[macro_export]
macro_rules! italic {
    ($e:expr) => {{
        let mut s = ::tui::text::Span::from($e);
        s.style = s.style.add_modifier(::tui::style::Modifier::ITALIC);
        s
    }};
}

/// styles text into a span with the underlined modifier set. The argument must evaluate to something
/// that implements [`Into<Span>`](tui::text::Span)
#[macro_export]
macro_rules! underlined {
    ($e:expr) => {{
        let mut s = ::tui::text::Span::from($e);
        s.style = s.style.add_modifier(::tui::style::Modifier::UNDERLINED);
        s
    }};
}

/// styles text into a span with the foreground set. The first argument must evaluate to something
/// that implements [`Into<Span>`](tui::text::Span), and the second a [`Color`](tui::style::Color)
#[macro_export]
macro_rules! fg {
    ($t:expr, $c: expr) => {{
        let mut s = ::tui::text::Span::from($t);
        s.style = s.style.fg($c);
        s
    }};
}

/// Styles text into a span with the background set. The first argument must evaluate to something
/// that implements [`Into<Span>`](tui::text::Span), and the second a [Color](tui::style::Color)
#[macro_export]
macro_rules! bg {
    ($t:expr, $c: expr) => {{
        let mut s = ::tui::text::Span::from($t);
        s.style = s.style.bg($c);
        s
    }};
}

/// Trait to allow all the overloading of the add_lines method
/// This is a helper to simplify the [text!](crate::text!) macro, and should not be used directly.
pub trait AddLines<T> {
    fn add_lines(&mut self, to_add: T);
}

impl<'a> AddLines<&'a str> for ::tui::text::Text<'a> {
    fn add_lines(&mut self, to_add: &'a str) {
        self.lines.push(to_add.into());
    }
}

impl<'a> AddLines<String> for ::tui::text::Text<'a> {
    fn add_lines(&mut self, to_add: String) {
        self.lines.push(to_add.into());
    }
}

impl<'a> AddLines<::tui::text::Span<'a>> for ::tui::text::Text<'a> {
    fn add_lines(&mut self, to_add: ::tui::text::Span<'a>) {
        self.lines.push(to_add.into());
    }
}

impl<'a> AddLines<::tui::text::Spans<'a>> for ::tui::text::Text<'a> {
    fn add_lines(&mut self, to_add: ::tui::text::Spans<'a>) {
        self.lines.push(to_add);
    }
}

impl<'a> AddLines<Vec<::tui::text::Spans<'a>>> for ::tui::text::Text<'a> {
    fn add_lines(&mut self, mut to_add: Vec<::tui::text::Spans<'a>>) {
        self.lines.append(&mut to_add);
    }
}

/// Create a [`Vec<Spans>`](tui::text::Spans) from lines of a string separated by '\n'
#[macro_export]
macro_rules! split {
    ($e:expr) => {{
        $e.lines()
            .map(|l| ::tui::text::Spans::from(l))
            .collect::<Vec<::tui::text::Spans>>()
    }};
}

/// Create a single [Spans](tui::text::Spans) from many
/// [Span](tui::text::Span) structs. Useful with [`text!`](crate::text!)
/// for having multiple stylings in a single line
#[macro_export]
macro_rules! line {
    ($($e:expr),* $(,)?) => {{
        let mut res = ::tui::text::Spans::default();
        $(res.0.push(::tui::text::Span::from($e));)*;
        res
    }};
}

/// Creates a `Vec<Spans>` from each line of the enclosed block
#[macro_export]
macro_rules! text {
    ($t:expr) => {
        res.push(Spans::from($t));
    };
    ($($t:expr);* $(;)?) => {{
        use $crate::text_macros::AddLines;
        let mut res = ::tui::text::Text::default();
        $(res.add_lines($t);)*
        res
    }};
}

#[cfg(test)]
mod tests {
    use tui::{
        style::{Modifier, Style},
        text::{Span, Spans, Text},
    };

    #[test]
    fn bold() {
        let expected = Span::styled("foo", Style::default().add_modifier(Modifier::BOLD));
        let test = bold!("foo");
        assert_eq!(expected, test);
    }

    #[test]
    fn italic() {
        let expected = Span::styled("foo", Style::default().add_modifier(Modifier::ITALIC));
        let test = italic!("foo");
        assert_eq!(expected, test);
    }

    #[test]
    fn underline() {
        let expected = Span::styled("foo", Style::default().add_modifier(Modifier::UNDERLINED));
        let test = underlined!("foo");
        assert_eq!(expected, test);
    }

    #[test]
    fn bold_italic() {
        let expected = Span::styled(
            "foo",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::ITALIC),
        );
        let test = bold!(italic!("foo"));
        assert_eq!(expected, test);
    }

    #[test]
    fn text() {
        let mut expected = Text::from(vec![
            Spans::from(Span::styled(
                "foo",
                Style::default().add_modifier(Modifier::ITALIC),
            )),
            Spans::from(Span::styled(
                "bar",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
            Spans::from("baz"),
        ]);

        let test = text! {
            italic!("foo");
            underlined!("bar");
            "baz";
        };
        assert_eq!(expected, test);

        let test = text! {
            italic!("foo");
            underlined!("bar");
            "baz"
        };
        assert_eq!(expected, test);

        let test = text! {
            italic!("foo");
            underlined!("bar");
            "baz";
            "a\nb";
            split!("q\nr")
        };
        expected.lines.push(Spans::from("a\nb"));
        expected.lines.push(Spans::from("q"));
        expected.lines.push(Spans::from("r"));
        assert_eq!(expected, test);
    }

    #[test]
    fn text_single_line() {
        let expected = Text::from(vec![Spans::from(Span::styled(
            "foo",
            Style::default().add_modifier(Modifier::ITALIC),
        ))]);

        let test = text! {
            italic!("foo");
        };

        assert_eq!(expected, test);
    }
}
