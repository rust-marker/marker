mod id;
pub use id::*;

use linter_api::ast::ty::Mutability;

use crate::context::RustcContext;

pub fn to_api_mutability(_cx: &RustcContext<'_, '_>, rustc_mt: rustc_ast::Mutability) -> Mutability {
    match rustc_mt {
        rustc_ast::Mutability::Mut => Mutability::Mut,
        rustc_ast::Mutability::Not => Mutability::Not,
    }
}
