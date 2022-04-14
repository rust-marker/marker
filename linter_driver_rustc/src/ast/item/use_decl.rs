use linter_api::ast::{
    item::{ExternCrateItem, GenericParam, ItemData, ItemType, UseDeclItem, UseKind, VisibilityKind},
    Path, Symbol,
};

use super::{RustcItem, RustcItemData};
use crate::ast::{rustc::RustcContext, RustcPath, ToApi};

use std::fmt::Debug;

#[derive(Debug)]
pub struct RustcUseDeclItemData<'ast, 'tcx> {
    use_kind: UseKind,
    path: RustcPath<'ast, 'tcx>, // FIXME: Add rustc path wrapper
}

pub type RustcUseDeclItem<'ast, 'tcx> = RustcItem<'ast, 'tcx, RustcUseDeclItemData<'ast, 'tcx>>;

impl<'ast, 'tcx> RustcItemData<'ast> for RustcUseDeclItem<'ast, 'tcx> {
    fn as_api_item(&'ast self) -> ItemType<'ast> {
        ItemType::UseDeclaration(self)
    }
}

impl<'ast, 'tcx> UseDeclItem<'ast> for RustcUseDeclItem<'ast, 'tcx> {
    fn get_path(&self) -> &dyn Path<'ast> {
        todo!()
    }

    fn get_use_kind(&self) -> UseKind {
        self.data.use_kind
    }
}

impl<'ast, 'tcx> RustcUseDeclItemData<'ast, 'tcx> {
    pub fn data_from_rustc(
        cx: &'ast RustcContext<'ast, 'tcx>,
        item: &'tcx rustc_hir::Item<'tcx>,
    ) -> Option<RustcUseDeclItemData<'ast, 'tcx>> {
        if let rustc_hir::ItemKind::Use(rustc_path, rustc_kind) = &item.kind {
            let use_kind = match rustc_kind {
                rustc_hir::UseKind::Single => UseKind::Single,
                rustc_hir::UseKind::Glob => UseKind::Glob,
                rustc_hir::UseKind::ListStem => return None,
            };

            let path = RustcPath::from_rustc(cx, rustc_path);

            Some(RustcUseDeclItemData { use_kind, path })
        } else {
            unreachable!()
        }
    }
}
