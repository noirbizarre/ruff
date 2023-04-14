use ruff_text_size::{TextLen, TextRange};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::newlines::Line;

use crate::rules::pycodestyle::helpers::is_overlong;
use crate::settings::Settings;

/// ## What it does
/// Checks for lines that exceed the specified maximum character length.
///
/// ## Why is this bad?
/// Overlong lines can hurt readability.
///
/// ## Example
/// ```python
/// my_function(param1, param2, param3, param4, param5, param6, param7, param8, param9, param10)
/// ```
///
/// Use instead:
/// ```python
/// my_function(
///     param1, param2, param3, param4, param5,
///     param6, param7, param8, param9, param10
/// )
/// ```
#[violation]
pub struct LineTooLong(pub usize, pub usize);

impl Violation for LineTooLong {
    #[derive_message_formats]
    fn message(&self) -> String {
        let LineTooLong(width, limit) = self;
        format!("Line too long ({width} > {limit} characters)")
    }
}

/// E501
pub(crate) fn line_too_long(line: &Line, settings: &Settings) -> Option<Diagnostic> {
    let line_width = line.width();
    let limit = settings.line_length;
    if is_overlong(
        line,
        line_width,
        limit,
        settings.pycodestyle.ignore_overlong_task_comments,
        &settings.task_tags,
    ) {
        let mut offset = line.end();
        let mut width = line_width;

        for c in line.chars() {
            width -= c.width().unwrap_or(0);
            offset -= c.text_len();

            if width == limit {
                break;
            }
        }

        Some(Diagnostic::new(
            LineTooLong(line_width, limit),
            TextRange::new(offset, line.end()),
        ))
    } else {
        None
    }
}
