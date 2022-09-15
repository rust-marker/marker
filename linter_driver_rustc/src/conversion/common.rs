mod id;
pub use id::*;
mod span;
pub use span::*;
mod unstable;
pub use unstable::*;

use linter_api::{ast::Mutability, lint::Level};

use crate::context::RustcContext;

pub fn to_api_mutability(_cx: &RustcContext<'_, '_>, rustc_mt: rustc_ast::Mutability) -> Mutability {
    match rustc_mt {
        rustc_ast::Mutability::Mut => Mutability::Mut,
        rustc_ast::Mutability::Not => Mutability::Not,
    }
}

pub fn to_rustc_lint_level(_cx: &RustcContext<'_, '_>, api_level: Level) -> rustc_lint::Level {
    match api_level {
        Level::Allow => rustc_lint::Level::Allow,
        Level::Warn => rustc_lint::Level::Warn,
        Level::Deny => rustc_lint::Level::Deny,
        Level::Forbid => rustc_lint::Level::Forbid,
        _ => unreachable!(),
    }
}
