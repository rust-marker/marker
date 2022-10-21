mod common;
pub use common::*;
use linter_api::ast::{item::ItemType, Crate};

use crate::context::RustcContext;

pub mod generic;
pub mod item;
pub mod ty;

pub fn to_api_crate<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    rustc_crate_id: rustc_hir::def_id::CrateNum,
    rustc_root_mod: &'tcx rustc_hir::Mod<'tcx>,
) -> &'ast Crate<'ast> {
    #[expect(
        clippy::needless_collect,
        reason = "collect is required to know the size of the allocation"
    )]
    let items: Vec<ItemType<'_>> = rustc_root_mod
        .item_ids
        .iter()
        .filter_map(|rustc_item| item::to_api_item(cx, cx.rustc_cx.hir().item(*rustc_item)))
        .collect();
    let items = cx.storage.alloc_slice_iter(items.into_iter());
    cx.storage
        .alloc(|| Crate::new(to_api_crate_id(cx, rustc_crate_id), items))
}
