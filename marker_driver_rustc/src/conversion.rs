mod common;
pub use common::*;
use marker_api::ast::Crate;

use crate::context::RustcContext;

use self::item::ItemConverter;

pub mod generic;
pub mod item;
pub mod rustc;
pub mod ty;

pub fn to_api_crate<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    rustc_crate_id: rustc_hir::def_id::CrateNum,
    rustc_root_mod: &'tcx rustc_hir::Mod<'tcx>,
) -> &'ast Crate<'ast> {
    let items = ItemConverter::new(cx);
    cx.storage
        .alloc(|| Crate::new(to_crate_id(rustc_crate_id), items.conv_items(rustc_root_mod.item_ids)))
}
