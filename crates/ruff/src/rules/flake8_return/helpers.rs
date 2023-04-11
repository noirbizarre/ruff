use ruff_text_size::TextLen;
use rustpython_parser::ast::{Constant, Expr, ExprKind, Location, Stmt};

use ruff_python_ast::newlines::StrExt;
use ruff_python_ast::source_code::Locator;

/// Return `true` if a function's return statement include at least one
/// non-`None` value.
pub fn result_exists(returns: &[(&Stmt, Option<&Expr>)]) -> bool {
    returns.iter().any(|(_, expr)| {
        expr.map(|expr| {
            !matches!(
                expr.node,
                ExprKind::Constant {
                    value: Constant::None,
                    ..
                }
            )
        })
        .unwrap_or(false)
    })
}

/// Given a statement, find its "logical end".
///
/// For example: the statement could be following by a trailing semicolon, by an end-of-line
/// comment, or by any number of continuation lines (and then by a comment, and so on).
///
/// This method assumes that the statement is the last statement in its body; specifically, that
/// the statement isn't followed by a semicolon, followed by a multi-line statement.
pub fn end_of_last_statement(stmt: &Stmt, locator: &Locator) -> Location {
    let contents = locator.after(stmt.end());

    // End-of-file, so just return the end of the statement.
    if contents.trim().is_empty() {
        return stmt.end();
    }

    // Otherwise, find the end of the last line that's "part of" the statement.
    let mut offset = stmt.end();
    for line in contents.universal_newlines() {
        offset += line.text_len();
        if line.ends_with('\\') {
            continue;
        }

        return offset;
    }

    unreachable!("Expected to find end-of-statement")
}
