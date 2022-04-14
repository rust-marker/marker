use linter_api::ast::{
    item::{ExternCrateItem, GenericParam, ItemData, ItemType, VisibilityKind},
    Symbol,
};

use super::{RustcItem, RustcItemData};
use crate::ast::{rustc::RustcContext, ToApi};

use std::fmt::Debug;

#[derive(Debug)]
pub struct RustcExternCrateItemData {
    original: Symbol,
}

pub type RustcExternCrateItem<'ast, 'tcx> = RustcItem<'ast, 'tcx, RustcExternCrateItemData>;

impl<'ast, 'tcx> RustcItemData<'ast> for RustcExternCrateItem<'ast, 'tcx> {
    fn as_api_item(&'ast self) -> ItemType<'ast> {
        ItemType::ExternCrate(self)
    }
}

impl<'ast, 'tcx> ExternCrateItem<'ast> for RustcExternCrateItem<'ast, 'tcx> {
    fn get_original_name(&self) -> Symbol {
        self.data.original
    }
}

impl<'ast, 'tcx> RustcExternCrateItem<'ast, 'tcx> {
    pub fn data_from_rustc(
        cx: &'ast RustcContext<'ast, 'tcx>,
        item: &'tcx rustc_hir::Item<'tcx>,
    ) -> RustcExternCrateItemData {
        if let rustc_hir::ItemKind::ExternCrate(opt_sym) = &item.kind {
            RustcExternCrateItemData {
                original: opt_sym.unwrap_or(item.ident.name).to_api(cx),
            }
        } else {
            unreachable!()
        }
    }
}
