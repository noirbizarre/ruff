use ruff_python_ast::newlines::{StrExt, UniversalNewlineIterator};
use ruff_text_size::{TextLen, TextRange, TextSize};
use std::fmt::{Debug, Formatter};
use std::iter::FusedIterator;
use strum_macros::EnumIter;

use crate::docstrings::definition::Docstring;
use crate::docstrings::styles::SectionStyle;
use ruff_python_ast::whitespace;

#[derive(EnumIter, PartialEq, Eq, Debug, Clone, Copy)]
pub enum SectionKind {
    Args,
    Arguments,
    Attention,
    Attributes,
    Caution,
    Danger,
    Error,
    Example,
    Examples,
    ExtendedSummary,
    Hint,
    Important,
    KeywordArgs,
    KeywordArguments,
    Methods,
    Note,
    Notes,
    OtherParameters,
    Parameters,
    Raises,
    References,
    Return,
    Returns,
    SeeAlso,
    ShortSummary,
    Tip,
    Todo,
    Warning,
    Warnings,
    Warns,
    Yield,
    Yields,
}

impl SectionKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "args" => Some(Self::Args),
            "arguments" => Some(Self::Arguments),
            "attention" => Some(Self::Attention),
            "attributes" => Some(Self::Attributes),
            "caution" => Some(Self::Caution),
            "danger" => Some(Self::Danger),
            "error" => Some(Self::Error),
            "example" => Some(Self::Example),
            "examples" => Some(Self::Examples),
            "extended summary" => Some(Self::ExtendedSummary),
            "hint" => Some(Self::Hint),
            "important" => Some(Self::Important),
            "keyword args" => Some(Self::KeywordArgs),
            "keyword arguments" => Some(Self::KeywordArguments),
            "methods" => Some(Self::Methods),
            "note" => Some(Self::Note),
            "notes" => Some(Self::Notes),
            "other parameters" => Some(Self::OtherParameters),
            "parameters" => Some(Self::Parameters),
            "raises" => Some(Self::Raises),
            "references" => Some(Self::References),
            "return" => Some(Self::Return),
            "returns" => Some(Self::Returns),
            "see also" => Some(Self::SeeAlso),
            "short summary" => Some(Self::ShortSummary),
            "tip" => Some(Self::Tip),
            "todo" => Some(Self::Todo),
            "warning" => Some(Self::Warning),
            "warnings" => Some(Self::Warnings),
            "warns" => Some(Self::Warns),
            "yield" => Some(Self::Yield),
            "yields" => Some(Self::Yields),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Args => "Args",
            Self::Arguments => "Arguments",
            Self::Attention => "Attention",
            Self::Attributes => "Attributes",
            Self::Caution => "Caution",
            Self::Danger => "Danger",
            Self::Error => "Error",
            Self::Example => "Example",
            Self::Examples => "Examples",
            Self::ExtendedSummary => "Extended Summary",
            Self::Hint => "Hint",
            Self::Important => "Important",
            Self::KeywordArgs => "Keyword Args",
            Self::KeywordArguments => "Keyword Arguments",
            Self::Methods => "Methods",
            Self::Note => "Note",
            Self::Notes => "Notes",
            Self::OtherParameters => "Other Parameters",
            Self::Parameters => "Parameters",
            Self::Raises => "Raises",
            Self::References => "References",
            Self::Return => "Return",
            Self::Returns => "Returns",
            Self::SeeAlso => "See Also",
            Self::ShortSummary => "Short Summary",
            Self::Tip => "Tip",
            Self::Todo => "Todo",
            Self::Warning => "Warning",
            Self::Warnings => "Warnings",
            Self::Warns => "Warns",
            Self::Yield => "Yield",
            Self::Yields => "Yields",
        }
    }
}

pub(crate) struct SectionContexts<'a> {
    contexts: Vec<SectionContextData>,
    docstring: &'a Docstring<'a>,
}

impl<'a> SectionContexts<'a> {
    /// Extract all `SectionContext` values from a docstring.
    pub fn from_docstring(docstring: &'a Docstring<'a>, style: SectionStyle) -> Self {
        let contents = docstring.body();

        let mut contexts = Vec::new();
        let mut last: Option<SectionContextData> = None;
        let mut offset = TextSize::default();

        for line in contents.universal_newlines() {
            if offset == TextSize::default() {
                // skip the first line
            } else if let Some(section_kind) = suspected_as_section(line, style) {
                let indent = whitespace::leading_space(line);
                let section_name = whitespace::leading_words(line);

                let section_range = TextRange::at(indent.text_len(), section_name.text_len());

                if is_docstring_section(line, section_range, &contents[TextRange::up_to(offset)]) {
                    if let Some(mut last) = last.take() {
                        last.range = TextRange::new(last.range.start(), offset);
                        contexts.push(last);
                    }

                    last = Some(SectionContextData {
                        kind: section_kind,
                        name_range: section_range + offset,
                        range: TextRange::empty(offset),
                    });
                }
            }

            offset += line.text_len();
        }

        if let Some(mut last) = last.take() {
            last.range = TextRange::new(last.range.start(), offset);
            contexts.push(last);
        }

        Self {
            docstring,
            contexts,
        }
    }

    pub fn len(&self) -> usize {
        self.contexts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.contexts.is_empty()
    }

    pub fn iter(&self) -> SectionContextsIter {
        SectionContextsIter {
            index: 0,
            sections: self,
        }
    }
}

impl<'a> IntoIterator for &'a SectionContexts<'a> {
    type Item = SectionContext<'a>;
    type IntoIter = SectionContextsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Debug for SectionContexts<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

pub struct SectionContextsIter<'a> {
    sections: &'a SectionContexts<'a>,
    index: usize,
}

impl<'a> Iterator for SectionContextsIter<'a> {
    type Item = SectionContext<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        if index < self.sections.contexts.len() {
            self.index += 1;
            Some(SectionContext {
                sections: self.sections,
                index,
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.sections.contexts.len() - self.index;
        (len, Some(len))
    }
}

impl FusedIterator for SectionContextsIter<'_> {}
impl ExactSizeIterator for SectionContextsIter<'_> {}

struct SectionContextData {
    kind: SectionKind,
    name_range: TextRange,
    range: TextRange,
}

#[derive(Debug)]
pub struct SectionContext<'a> {
    sections: &'a SectionContexts<'a>,
    index: usize,
}

impl<'a> SectionContext<'a> {
    pub fn is_last(&self) -> bool {
        self.index == self.sections.contexts.len() - 1
    }

    fn data(&self) -> &SectionContextData {
        &self.sections.contexts[self.index]
    }

    /// The "kind" of the section, e.g. "SectionKind::Args" or "SectionKind::Returns".
    pub fn kind(&self) -> SectionKind {
        self.data().kind
    }

    /// The name  of the section as it appears in the docstring, e.g. "Args" or "Returns".
    pub fn section_name(&self) -> &'a str {
        &self.sections.docstring.body().as_str()[self.data().name_range]
    }

    /// Range of this sections text relative to [`Docstring.body`].
    fn relative_range(&self) -> TextRange {
        self.data().range
    }

    /// Absolute range
    pub fn range(&self) -> TextRange {
        self.data().range + self.sections.docstring.body().start()
    }

    pub fn summary_after_section_name(&self) -> &'a str {
        &self.sections.docstring.body().as_str()
            [TextRange::new(self.data().name_range.end(), self.summary_line().text_len())]
    }

    pub fn section_name_range(&self) -> TextRange {
        self.data().name_range + self.range().start()
    }

    pub fn header_range(&self) -> TextRange {
        TextRange::at(self.range().start(), self.summary_line().text_len())
    }

    pub fn text(&self) -> &'a str {
        &self.sections.docstring.body().as_str()[self.relative_range()]
    }

    pub fn summary_line(&self) -> &'a str {
        let end = self
            .text()
            .find(['\n', '\r'])
            .unwrap_or_else(|| self.text().len());
        &self.text()[..end]
    }

    pub fn previous_line(&self) -> &'a str {
        let previous = &self.sections.docstring.body().as_str()
            [TextRange::up_to(self.relative_range().start())];
        previous.universal_newlines().last().unwrap_or_default()
    }

    pub fn following_lines(&self) -> UniversalNewlineIterator<'a> {
        let after = &self.sections.docstring.body().as_str()[self.following_relative_range()];
        after.universal_newlines()
    }

    /// Returns the range to the following lines relative to [`Docstring.body`].
    fn following_relative_range(&self) -> TextRange {
        let end = self
            .sections
            .contexts
            .get(self.index + 1)
            .map(|data| data.range.start())
            .unwrap_or_else(|| self.sections.docstring.body().text_len());

        TextRange::new(self.relative_range().end(), end)
    }

    /// Returns the absolute range to the following lines.
    pub fn following_range(&self) -> TextRange {
        self.following_relative_range() + self.range().start()
    }
}

fn suspected_as_section(line: &str, style: SectionStyle) -> Option<SectionKind> {
    if let Some(kind) = SectionKind::from_str(whitespace::leading_words(line)) {
        if style.sections().contains(&kind) {
            return Some(kind);
        }
    }
    None
}

/// Check if the suspected context is really a section header.
fn is_docstring_section(line: &str, section_range: TextRange, previous_lines: &str) -> bool {
    let section_name_suffix = line[usize::from(section_range.end())..].trim();
    let this_looks_like_a_section_name =
        section_name_suffix == ":" || section_name_suffix.is_empty();
    if !this_looks_like_a_section_name {
        return false;
    }

    let prev_line = previous_lines.trim();
    let prev_line_ends_with_punctuation = [',', ';', '.', '-', '\\', '/', ']', '}', ')']
        .into_iter()
        .any(|char| prev_line.ends_with(char));
    let prev_line_looks_like_end_of_paragraph =
        prev_line_ends_with_punctuation || prev_line.is_empty();
    if !prev_line_looks_like_end_of_paragraph {
        return false;
    }

    true
}
