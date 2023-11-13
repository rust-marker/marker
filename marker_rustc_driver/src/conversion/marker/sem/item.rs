use marker_api::prelude::*;
use rustc_hir as hir;
use rustc_middle as mid;

use crate::conversion::marker::MarkerConverterInner;

impl<'ast, 'tcx> MarkerConverterInner<'ast, 'tcx> {
    pub fn to_sem_visibility(&self, owner_id: hir::def_id::LocalDefId, has_span: bool) -> sem::Visibility<'ast> {
        let vis = self.rustc_cx.visibility(owner_id);
        let kind = match vis {
            mid::ty::Visibility::Public => sem::VisibilityKind::Public,
            mid::ty::Visibility::Restricted(id) if id == hir::def_id::CRATE_DEF_ID.to_def_id() => {
                if has_span {
                    sem::VisibilityKind::Crate(self.to_item_id(id))
                } else {
                    sem::VisibilityKind::DefaultCrate(self.to_item_id(id))
                }
            },
            mid::ty::Visibility::Restricted(id) => {
                if has_span {
                    sem::VisibilityKind::Path(self.to_item_id(id))
                } else {
                    sem::VisibilityKind::Default(self.to_item_id(id))
                }
            },
        };

        sem::Visibility::builder().kind(kind).build()
    }
}
