use linter_api::ast::{
    item::{ExternCrateItem, GenericParam, ItemData, ItemType, UseDeclItem, UseKind, VisibilityKind},
    Path, Symbol,
};

use super::{RustcItem, RustcItemData};
use crate::ast::{path_from_rustc, rustc::RustcContext, ToApi};

use std::fmt::Debug;

#[derive(Debug)]
pub struct RustcUseDeclItemData<'ast> {
    use_kind: UseKind,
    path: Path<'ast>,
}

pub type RustcUseDeclItem<'ast, 'tcx> = RustcItem<'ast, 'tcx, RustcUseDeclItemData<'ast>>;

impl<'ast, 'tcx> RustcItemData<'ast> for RustcUseDeclItem<'ast, 'tcx> {
    fn as_api_item(&'ast self) -> ItemType<'ast> {
        ItemType::UseDeclaration(self)
    }
}

impl<'ast, 'tcx> UseDeclItem<'ast> for RustcUseDeclItem<'ast, 'tcx> {
    fn get_path(&self) -> &Path<'ast> {
        &self.data.path
    }

    fn get_use_kind(&self) -> UseKind {
        self.data.use_kind
    }
}

impl<'ast, 'tcx> RustcUseDeclItemData<'ast> {
    pub fn data_from_rustc(
        cx: &'ast RustcContext<'ast, 'tcx>,
        item: &'tcx rustc_hir::Item<'tcx>,
    ) -> Option<RustcUseDeclItemData<'ast>> {
        if let rustc_hir::ItemKind::Use(rustc_path, rustc_kind) = &item.kind {
            let use_kind = match rustc_kind {
                rustc_hir::UseKind::Single => UseKind::Single,
                rustc_hir::UseKind::Glob => UseKind::Glob,
                rustc_hir::UseKind::ListStem => return None,
            };

            let path = path_from_rustc(cx, rustc_path);

            Some(RustcUseDeclItemData { use_kind, path })
        } else {
            unreachable!()
        }
    }
}
