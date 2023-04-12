use ruff_text_size::{TextLen, TextRange, TextSize};

use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::whitespace::leading_space;

#[violation]
pub struct TabIndentation;

impl Violation for TabIndentation {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Indentation contains tabs")
    }
}

/// W191
pub fn tab_indentation(
    line_range: TextRange,
    line: &str,
    string_ranges: &[TextRange],
) -> Option<Diagnostic> {
    let indent = leading_space(line);
    if let Some(tab_index) = indent.find('\t') {
        let tab_offset = line_range.start() + TextSize::try_from(tab_index).unwrap();

        // If the tab character is within a multi-line string, abort.
        if let Ok(_) = string_ranges.binary_search_by(|range| {
            if tab_offset < range.start() {
                std::cmp::Ordering::Less
            } else if range.contains(tab_offset) {
                std::cmp::Ordering::Equal
            } else {
                std::cmp::Ordering::Greater
            }
        }) {
            None
        } else {
            Some(Diagnostic::new(
                TabIndentation,
                TextRange::at(line_range.start(), indent.text_len()),
            ))
        }
    } else {
        None
    }
}
