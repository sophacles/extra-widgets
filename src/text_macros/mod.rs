//! macros for building and styling text for tui.

/// styles text into a span with the bold modifier set. The argument must evaluate to something
/// that implements Into<Span>
#[macro_export]
macro_rules! bold {
    ($e:expr) => {{
        let mut s = ::tui::text::Span::from($e);
        s.style = s.style.add_modifier(::tui::style::Modifier::BOLD);
        s
    }};
}

/// styles text into a span with the italic modifier set. The argument must evaluate to something
/// that implements Into<Span>
#[macro_export]
macro_rules! italic {
    ($e:expr) => {{
        let mut s = ::tui::text::Span::from($e);
        s.style = s.style.add_modifier(::tui::style::Modifier::ITALIC);
        s
    }};
}

/// styles text into a span with the underlined modifier set. The argument must evaluate to something
/// that implements Into<Span>
#[macro_export]
macro_rules! underlined {
    ($e:expr) => {{
        let mut s = ::tui::text::Span::from($e);
        s.style = s.style.add_modifier(::tui::style::Modifier::UNDERLINED);
        s
    }};
}

/// Creates a Vec<Spans> from each line of the enclosed block
#[macro_export]
macro_rules! text {
    ($t:expr) => {
        res.push(Spans::from($t));
    };
    ($($t:expr);* $(;)?) => {{
        let mut res = Vec::new();
        $(res.push(Spans::from($t));)*
        res

    }};
}

#[cfg(test)]
mod tests {
    use tui::{
        style::{Color, Modifier, Style},
        text::{Span, Spans},
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
        let mut expected = Vec::new();
        expected.push(Spans::from(Span::styled(
            "foo",
            Style::default().add_modifier(Modifier::ITALIC),
        )));
        expected.push(Spans::from(Span::styled(
            "bar",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )));

        let test = text! {
            italic!("foo");
            underlined!("bar");
        };

        assert_eq!(expected, test);
    }

    #[test]
    fn text_single_line() {
        let mut expected = Vec::new();
        expected.push(Spans::from(Span::styled(
            "foo",
            Style::default().add_modifier(Modifier::ITALIC),
        )));

        let test = text! {
            italic!("foo");
        };

        assert_eq!(expected, test);
    }
}
