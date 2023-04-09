use once_cell::sync::Lazy;
use regex::Regex;
use ruff_text_size::TextRange;
use rustpython_parser::ast::Location;

use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};

#[violation]
pub struct BlanketNOQA;

impl Violation for BlanketNOQA {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Use specific rule codes when using `noqa`")
    }
}

static BLANKET_NOQA_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)# noqa($|\s|:[^ ])").unwrap());

/// PGH004
pub fn blanket_noqa(diagnostics: &mut Vec<Diagnostic>, lineno: usize, line: &str) {
    if let Some(match_) = BLANKET_NOQA_REGEX.find(line) {
        let start = line[..match_.start()].chars().count();
        let end = start + line[match_.start()..match_.end()].chars().count();
        diagnostics.push(Diagnostic::new(
            BlanketNOQA,
            TextRange::new(
                Location::new(lineno + 1, start),
                Location::new(lineno + 1, end),
            ),
        ));
    }
}
