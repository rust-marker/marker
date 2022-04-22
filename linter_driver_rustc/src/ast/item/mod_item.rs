use linter_api::ast::{
    item::{ExternCrateItem, GenericParam, ItemData, ItemType, ModItem, VisibilityKind},
    Symbol,
};

use super::{from_rustc, RustcItem, RustcItemData};
use crate::ast::{rustc::RustcContext, ToApi};

use std::fmt::Debug;

#[derive(Debug)]
pub struct RustcModItemData<'ast> {
    items: &'ast [ItemType<'ast>],
}

pub type RustcModItem<'ast, 'tcx> = RustcItem<'ast, 'tcx, RustcModItemData<'ast>>;

impl<'ast, 'tcx> RustcItemData<'ast> for RustcModItem<'ast, 'tcx> {
    fn as_api_item(&'ast self) -> ItemType<'ast> {
        ItemType::Mod(self)
    }
}

impl<'ast, 'tcx> ModItem<'ast> for RustcModItem<'ast, 'tcx> {
    fn get_inner_attrs(&self) {
        todo!()
    }

    fn get_items(&self) -> &[ItemType<'ast>] {
        self.data.items
    }
}

impl<'ast, 'tcx> RustcModItem<'ast, 'tcx> {
    pub fn data_from_rustc(
        cx: &'ast RustcContext<'ast, 'tcx>,
        item: &'tcx rustc_hir::Item<'tcx>,
    ) -> RustcModItemData<'ast> {
        if let rustc_hir::ItemKind::Mod(rustc_mod) = &item.kind {
            #[allow(clippy::needless_collect, reason = "collect is required to know the size of the allocation")]
            let items: Vec<ItemType<'_>> = rustc_mod
                .item_ids
                .iter().filter_map(|rustc_item| from_rustc(cx, cx.tcx.hir().item(*rustc_item)))
                .collect();
            let items = cx.alloc_slice_from_iter(items.into_iter());
            RustcModItemData { items }
        } else {
            unreachable!()
        }
    }
}
