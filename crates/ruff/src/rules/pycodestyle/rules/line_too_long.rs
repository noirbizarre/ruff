use ruff_text_size::{TextRange, TextSize};
use unicode_width::UnicodeWidthStr;

use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};

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
pub fn line_too_long(line_range: TextRange, line: &str, settings: &Settings) -> Option<Diagnostic> {
    let line_width = line.width();
    let limit = settings.line_length;
    if is_overlong(
        line,
        line_width,
        limit,
        settings.pycodestyle.ignore_overlong_task_comments,
        &settings.task_tags,
    ) {
        Some(Diagnostic::new(
            LineTooLong(line_width, limit),
            TextRange::new(
                line_range.start() + TextSize::try_from(limit).unwrap(),
                line_range.end(),
            ),
        ))
    } else {
        None
    }
}
