mod common;
pub use common::*;

use self::rustc::RustcContext;

pub mod item;
pub mod rustc;
pub mod ty;

/// This trait is used to have one common method for all api types that can be
/// converted with a simple `to_api`
pub trait ToApi<'ast, 'tcx, T> {
    fn to_api(&self, _cx: &'ast RustcContext<'ast, 'tcx>) -> T;
}
