#![expect(unused)]

use linter_api::ast::{
    item::{
        CommonItemData, ExternCrateItem, GenericParam, ItemData, ItemId, ItemType, ModItem, StaticItem, UseDeclItem,
        UseKind, Visibility,
    },
    CrateId, Symbol,
};

use super::{path_from_rustc, rustc::RustcContext};
use crate::ast::ToApi;

use std::{fmt::Debug, mem::transmute};

impl<'ast, 'tcx> ToApi<'ast, 'tcx, ItemId> for rustc_hir::def_id::DefId {
    fn to_api(&self, cx: &'ast RustcContext<'ast, 'tcx>) -> ItemId {
        ItemId::new(self.krate.to_api(cx), self.index.as_u32())
    }
}

#[must_use]
pub fn rustc_item_id_from_api_item_id(api_id: ItemId) -> rustc_hir::def_id::DefId {
    let (krate, index) = api_id.get_data();
    rustc_hir::def_id::DefId {
        index: unsafe { std::mem::transmute::<u32, rustc_hir::def_id::DefIndex>(index) },
        krate: rustc_crate_id_from_api_create_id(krate),
    }
}

#[must_use]
pub fn rustc_crate_id_from_api_create_id(api_id: CrateId) -> rustc_hir::def_id::CrateNum {
    unsafe { transmute::<CrateId, rustc_hir::def_id::CrateNum>(api_id) }
}

#[cfg(test)]
pub mod test {
    #[allow(clippy::missing_panics_doc)]
    #[test]
    pub fn test_magic_sizes() {
        assert_eq!(std::mem::size_of::<rustc_hir::def_id::DefIndex>(), 4);
        assert_eq!(std::mem::size_of::<rustc_hir::def_id::CrateNum>(), 4);
        assert_eq!(std::mem::size_of::<linter_api::ast::CrateId>(), 4);
    }
}

pub fn from_rustc<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    item: &'tcx rustc_hir::Item<'tcx>,
) -> Option<ItemType<'ast>> {
    Some(match &item.kind {
        rustc_hir::ItemKind::Mod(rustc_mod) => ItemType::Mod(cx.alloc_with(|| {
            #[expect(
                clippy::needless_collect,
                reason = "collect is required to know the size of the allocation"
            )]
            let items: Vec<ItemType<'_>> = rustc_mod
                .item_ids
                .iter()
                .filter_map(|rustc_item| from_rustc(cx, cx.rustc_cx.tcx.hir().item(*rustc_item)))
                .collect();
            let items = cx.alloc_slice_from_iter(items.into_iter());
            ModItem::new(create_common_data(cx, item), items)
        })),
        rustc_hir::ItemKind::ExternCrate(rustc_original_name) => ItemType::ExternCrate(cx.alloc_with(|| {
            let original_name = rustc_original_name.unwrap_or(item.ident.name).to_api(cx);
            ExternCrateItem::new(create_common_data(cx, item), original_name)
        })),
        rustc_hir::ItemKind::Use(rustc_path, rustc_use_kind) => {
            let use_kind = rustc_use_kind.to_api(cx)?;
            ItemType::UseDecl(cx.alloc_with(|| {
                UseDeclItem::new(create_common_data(cx, item), path_from_rustc(cx, rustc_path), use_kind)
            }))
        },
        rustc_hir::ItemKind::Static(_ty, rustc_mut, rustc_body_id) => ItemType::Static(cx.alloc_with(|| {
            StaticItem::new(
                create_common_data(cx, item),
                rustc_mut.to_api(cx),
                rustc_body_id.to_api(cx),
            )
        })),
        rustc_hir::ItemKind::Const(..)
        | rustc_hir::ItemKind::Fn(..)
        | rustc_hir::ItemKind::Macro(..)
        | rustc_hir::ItemKind::ForeignMod { .. }
        | rustc_hir::ItemKind::GlobalAsm(..)
        | rustc_hir::ItemKind::TyAlias(..)
        | rustc_hir::ItemKind::OpaqueTy(..)
        | rustc_hir::ItemKind::Enum(..)
        | rustc_hir::ItemKind::Struct(..)
        | rustc_hir::ItemKind::Union(..)
        | rustc_hir::ItemKind::Trait(..)
        | rustc_hir::ItemKind::TraitAlias(..)
        | rustc_hir::ItemKind::Impl(..) => return None,
    })
}

fn create_common_data<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    rustc_item: &'tcx rustc_hir::Item<'tcx>,
) -> CommonItemData<'ast> {
    CommonItemData::new(
        cx.ast_cx(),
        rustc_item.def_id.to_def_id().to_api(cx),
        vis_from_rustc(cx, rustc_item),
        (!rustc_item.ident.name.is_empty()).then(|| rustc_item.ident.name.to_api(cx)),
    )
}

fn vis_from_rustc<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    item: &'tcx rustc_hir::Item<'tcx>,
) -> Visibility<'ast> {
    match cx.rustc_cx.tcx.visibility(item.def_id) {
        rustc_middle::ty::Visibility::Public => Visibility::Pub,
        rustc_middle::ty::Visibility::Restricted(rustc_def_id)
            if rustc_def_id == rustc_hir::def_id::CRATE_DEF_ID.to_def_id() =>
        {
            Visibility::PubCrate
        },
        // FIXME: Fix visibility conversion. See #26
        rustc_middle::ty::Visibility::Restricted(_) | rustc_middle::ty::Visibility::Invisible => Visibility::None,
    }
}

impl<'ast, 'tcx> ToApi<'ast, 'tcx, Option<UseKind>> for rustc_hir::UseKind {
    fn to_api(&self, cx: &'ast RustcContext<'ast, 'tcx>) -> Option<UseKind> {
        match self {
            rustc_hir::UseKind::Single => Some(UseKind::Single),
            rustc_hir::UseKind::Glob => Some(UseKind::Glob),
            rustc_hir::UseKind::ListStem => None,
        }
    }
}
