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

use std::fmt::Debug;

impl<'ast, 'tcx> ToApi<'ast, 'tcx, ItemId> for rustc_hir::def_id::DefId {
    fn to_api(&self, cx: &'ast RustcContext<'ast, 'tcx>) -> ItemId {
        ItemId::new(self.krate.to_api(cx), self.index.as_u32())
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
        rustc_hir::ItemKind::Const(..) => None?,
        rustc_hir::ItemKind::Fn(..) => None?,
        rustc_hir::ItemKind::Macro(..) => None?,
        rustc_hir::ItemKind::ForeignMod { .. } => None?,
        rustc_hir::ItemKind::GlobalAsm(..) => None?,
        rustc_hir::ItemKind::TyAlias(..) => None?,
        rustc_hir::ItemKind::OpaqueTy(..) => None?,
        rustc_hir::ItemKind::Enum(..) => None?,
        rustc_hir::ItemKind::Struct(..) => None?,
        rustc_hir::ItemKind::Union(..) => None?,
        rustc_hir::ItemKind::Trait(..) => None?,
        rustc_hir::ItemKind::TraitAlias(..) => None?,
        rustc_hir::ItemKind::Impl(..) => None?,
    })
}

fn create_common_data<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    rustc_item: &'tcx rustc_hir::Item<'tcx>,
) -> CommonItemData<'ast> {
    CommonItemData::new(
        rustc_item.def_id.to_def_id().to_api(cx),
        rustc_item.span.to_api(cx),
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
        rustc_middle::ty::Visibility::Invisible => Visibility::None,
        _ => Visibility::None, // FIXME: Fix visibility conversion. See #26
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
