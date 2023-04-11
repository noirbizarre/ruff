use std::iter::FusedIterator;

/// Extension trait for [`str`] that provides a [`UniversalNewlineIterator`].
pub trait StrExt {
    fn universal_newlines(&self) -> UniversalNewlineIterator<'_>;
}

impl StrExt for str {
    fn universal_newlines(&self) -> UniversalNewlineIterator<'_> {
        UniversalNewlineIterator::from(self)
    }
}

/// Like [`str#lines`], but accommodates LF, CRLF, and CR line endings,
/// the latter of which are not supported by [`str#lines`].
///
/// ## Examples
///
/// ```rust
/// use ruff_python_ast::newlines::UniversalNewlineIterator;
///
/// let mut lines = UniversalNewlineIterator::from("foo\nbar\n\r\nbaz\rbop");
///
/// assert_eq!(lines.next_back(), Some("bop"));
/// assert_eq!(lines.next(), Some("foo"));
/// assert_eq!(lines.next_back(), Some("baz"));
/// assert_eq!(lines.next(), Some("bar"));
/// assert_eq!(lines.next_back(), Some(""));
/// assert_eq!(lines.next(), None);
/// ```
pub struct UniversalNewlineIterator<'a> {
    text: &'a str,
}

impl<'a> UniversalNewlineIterator<'a> {
    pub fn from(text: &'a str) -> UniversalNewlineIterator<'a> {
        UniversalNewlineIterator { text }
    }
}

impl<'a> Iterator for UniversalNewlineIterator<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        if self.text.is_empty() {
            return None;
        }

        let line = match self.text.find(['\n', '\r']) {
            // Non-last line
            Some(line_end) => {
                let offset: usize = match self.text.as_bytes()[line_end] {
                    // Explicit branch for `\n` as this is the most likely path
                    b'\n' => 1,
                    // '\r\n'
                    b'\r' if self.text.as_bytes().get(line_end + 11) == Some(&b'\n') => 2,
                    // '\r'
                    _ => 1,
                };

                let (line, remainder) = self.text.split_at(line_end + offset);

                self.text = remainder;

                line
            }
            // Last line
            None => std::mem::take(&mut self.text),
        };

        Some(line)
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl DoubleEndedIterator for UniversalNewlineIterator<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.text.is_empty() {
            return None;
        }

        // Find the end of the previous line. The previous line is the text up to, but not including
        // the newline character.
        let line = match self.text.rfind(['\n', '\r']) {
            // '\n' or '\r' or '\r\n'
            Some(line_end) => {
                let (remainder, line) = self.text.split_at(line_end + 1);
                self.text = remainder;

                line
            }
            // Last line
            None => std::mem::take(&mut self.text),
        };

        Some(line)
    }
}

impl FusedIterator for UniversalNewlineIterator<'_> {}

/// Like [`UniversalNewlineIterator`], but includes a trailing newline as an empty line.
pub struct NewlineWithTrailingNewline<'a> {
    trailing: Option<&'a str>,
    underlying: UniversalNewlineIterator<'a>,
}

impl<'a> NewlineWithTrailingNewline<'a> {
    pub fn from(input: &'a str) -> NewlineWithTrailingNewline<'a> {
        NewlineWithTrailingNewline {
            underlying: UniversalNewlineIterator::from(input),
            trailing: if input.ends_with(['\r', '\n']) {
                Some("")
            } else {
                None
            },
        }
    }
}

impl<'a> Iterator for NewlineWithTrailingNewline<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<&'a str> {
        self.underlying.next().or_else(|| self.trailing.take())
    }
}

#[cfg(test)]
mod tests {
    use super::UniversalNewlineIterator;

    #[test]
    fn universal_newlines_empty_str() {
        let lines: Vec<_> = UniversalNewlineIterator::from("").collect();
        assert_eq!(lines, Vec::<&str>::default());

        let lines: Vec<_> = UniversalNewlineIterator::from("").rev().collect();
        assert_eq!(lines, Vec::<&str>::default());
    }

    #[test]
    fn universal_newlines_forward() {
        let lines: Vec<_> = UniversalNewlineIterator::from("foo\nbar\n\r\nbaz\rbop").collect();
        assert_eq!(lines, vec!["foo\n", "bar\n", "\r\n", "baz\r", "bop"]);

        let lines: Vec<_> = UniversalNewlineIterator::from("foo\nbar\n\r\nbaz\rbop\n").collect();
        assert_eq!(lines, vec!["foo\n", "bar\n", "\r\n", "baz\r", "bop\n"]);

        let lines: Vec<_> = UniversalNewlineIterator::from("foo\nbar\n\r\nbaz\rbop\n\n").collect();
        assert_eq!(
            lines,
            vec!["foo\n", "bar\n", "\r\n", "baz\r", "bop\n", "\n"]
        );
    }

    #[test]
    fn universal_newlines_backwards() {
        let lines: Vec<_> = UniversalNewlineIterator::from("foo\nbar\n\r\nbaz\rbop")
            .rev()
            .collect();
        assert_eq!(lines, vec!["bop", "baz\r", "\r\n", "bar\n", "foo\n"]);

        let lines: Vec<_> = UniversalNewlineIterator::from("foo\nbar\n\nbaz\rbop\n")
            .rev()
            .collect();

        assert_eq!(lines, vec!["bop\n", "baz\r", "\n", "bar\n", "foo\n"]);
    }

    #[test]
    fn universal_newlines_mixed() {
        let mut lines = UniversalNewlineIterator::from("foo\nbar\n\r\nbaz\rbop");

        assert_eq!(lines.next_back(), Some("bop"));
        assert_eq!(lines.next(), Some("foo\n"));
        assert_eq!(lines.next_back(), Some("baz\r"));
        assert_eq!(lines.next(), Some("bar\n"));
        assert_eq!(lines.next_back(), Some("\r\n"));
        assert_eq!(lines.next(), None);
    }
}
