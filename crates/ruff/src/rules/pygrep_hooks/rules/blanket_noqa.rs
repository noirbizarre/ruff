use once_cell::sync::Lazy;
use regex::Regex;
use ruff_text_size::{TextLen, TextRange, TextSize};

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
pub fn blanket_noqa(diagnostics: &mut Vec<Diagnostic>, line_range: TextRange, line: &str) {
    if let Some(match_) = BLANKET_NOQA_REGEX.find(line) {
        diagnostics.push(Diagnostic::new(
            BlanketNOQA,
            TextRange::at(
                line_range.start() + TextSize::try_from(match_.start()).unwrap(),
                match_.as_str().text_len(),
            ),
        ));
    }
}
