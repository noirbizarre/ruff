use ruff_text_size::{TextRange, TextSize};
use rustpython_parser::ast::{Expr, Stmt};
use std::ops::Deref;

use ruff_python_semantic::analyze::visibility::{
    class_visibility, function_visibility, method_visibility, Modifier, Visibility, VisibleScope,
};

#[derive(Debug, Clone)]
pub enum DefinitionKind<'a> {
    Module,
    Package,
    Class(&'a Stmt),
    NestedClass(&'a Stmt),
    Function(&'a Stmt),
    NestedFunction(&'a Stmt),
    Method(&'a Stmt),
}

#[derive(Debug)]
pub struct Definition<'a> {
    pub kind: DefinitionKind<'a>,
    pub docstring: Option<&'a Expr>,
}

#[derive(Debug)]
pub struct Docstring<'a> {
    pub kind: DefinitionKind<'a>,
    pub expr: &'a Expr,
    pub contents: &'a str,
    pub body_range: TextRange,
    pub indentation: &'a str,
}

pub struct DocstringBody<'a> {
    docstring: &'a Docstring<'a>,
}

impl<'a> Docstring<'a> {
    pub fn body(&self) -> DocstringBody {
        DocstringBody { docstring: &self }
    }

    pub const fn start(&self) -> TextSize {
        self.expr.start()
    }

    pub const fn end(&self) -> TextSize {
        self.expr.end()
    }

    pub const fn range(&self) -> TextRange {
        self.expr.range()
    }

    pub fn leading_quote(&self) -> &'a str {
        &self.contents[TextRange::new(self.start(), self.body_range.start())]
    }

    pub fn trailing_quote(&self) -> &'a str {
        &self.contents[TextRange::new(self.body_range.end(), self.end())]
    }
}

impl<'a> DocstringBody<'a> {
    pub const fn start(&self) -> TextSize {
        self.docstring.body_range.start()
    }

    pub const fn end(&self) -> TextSize {
        self.docstring.body_range.end()
    }

    pub fn as_str(&self) -> &'a str {
        &self.docstring.contents[self.docstring.body_range]
    }
}

impl Deref for DocstringBody<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

#[derive(Copy, Clone)]
pub enum Documentable {
    Class,
    Function,
}

pub fn transition_scope(scope: VisibleScope, stmt: &Stmt, kind: Documentable) -> VisibleScope {
    match kind {
        Documentable::Function => VisibleScope {
            modifier: Modifier::Function,
            visibility: match scope {
                VisibleScope {
                    modifier: Modifier::Module,
                    visibility: Visibility::Public,
                } => function_visibility(stmt),
                VisibleScope {
                    modifier: Modifier::Class,
                    visibility: Visibility::Public,
                } => method_visibility(stmt),
                _ => Visibility::Private,
            },
        },
        Documentable::Class => VisibleScope {
            modifier: Modifier::Class,
            visibility: match scope {
                VisibleScope {
                    modifier: Modifier::Module,
                    visibility: Visibility::Public,
                } => class_visibility(stmt),
                VisibleScope {
                    modifier: Modifier::Class,
                    visibility: Visibility::Public,
                } => class_visibility(stmt),
                _ => Visibility::Private,
            },
        },
    }
}
