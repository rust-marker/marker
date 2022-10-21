use linter_api::ast::item::CommonItemData;
use linter_api::ast::item::ItemType;
use linter_api::ast::item::ModItem;
use linter_api::ast::item::StaticItem;
use linter_api::ast::item::Visibility;

use crate::context::RustcContext;

use super::to_api_body_id;
use super::to_api_item_id_from_def_id;
use super::to_api_mutability;
use super::to_api_symbol_id;
use super::ty::to_api_syn_ty;

pub fn to_api_item<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    rustc_item: &'tcx rustc_hir::Item<'tcx>,
) -> Option<ItemType<'ast>> {
    let id = to_api_item_id_from_def_id(cx, rustc_item.def_id.to_def_id());
    if let Some(item) = cx.storage.item(id) {
        return Some(item);
    }

    let common_data = CommonItemData::new(
        cx.ast_cx(),
        id,
        Visibility::new(cx.ast_cx(), id),
        to_api_symbol_id(cx, rustc_item.ident.name),
    );
    let item = match rustc_item.kind {
        rustc_hir::ItemKind::Mod(rustc_mod) => ItemType::Mod(to_mod_item(cx, common_data, rustc_mod)),
        rustc_hir::ItemKind::Static(ty, mt, rust_body_id) => {
            ItemType::Static(to_static_item(cx, common_data, ty, mt, rust_body_id))
        },
        rustc_hir::ItemKind::ExternCrate(_)
        | rustc_hir::ItemKind::Use(_, _)
        | rustc_hir::ItemKind::Const(_, _)
        | rustc_hir::ItemKind::Fn(_, _, _)
        | rustc_hir::ItemKind::Macro(_, _)
        | rustc_hir::ItemKind::ForeignMod { .. }
        | rustc_hir::ItemKind::GlobalAsm(_)
        | rustc_hir::ItemKind::TyAlias(_, _)
        | rustc_hir::ItemKind::OpaqueTy(_)
        | rustc_hir::ItemKind::Enum(_, _)
        | rustc_hir::ItemKind::Struct(_, _)
        | rustc_hir::ItemKind::Union(_, _)
        | rustc_hir::ItemKind::Trait(_, _, _, _, _)
        | rustc_hir::ItemKind::TraitAlias(_, _)
        | rustc_hir::ItemKind::Impl(_) => None?,
    };

    cx.storage.add_item(id, item);
    Some(item)
}

fn to_mod_item<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    data: CommonItemData<'ast>,
    rustc_mod: &rustc_hir::Mod,
) -> &'ast ModItem<'ast> {
    #[expect(
        clippy::needless_collect,
        reason = "collect is required to know the size of the allocation"
    )]
    let items: Vec<ItemType<'_>> = rustc_mod
        .item_ids
        .iter()
        .filter_map(|rustc_item| to_api_item(cx, cx.rustc_cx.hir().item(*rustc_item)))
        .collect();
    let items = cx.storage.alloc_slice_iter(items.into_iter());
    cx.storage.alloc(|| ModItem::new(data, items))
}

fn to_static_item<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    data: CommonItemData<'ast>,
    ty: &'tcx rustc_hir::Ty<'tcx>,
    rustc_mt: rustc_ast::Mutability,
    rustc_body_id: rustc_hir::BodyId,
) -> &'ast StaticItem<'ast> {
    cx.storage.alloc(|| {
        StaticItem::new(
            data,
            to_api_mutability(cx, rustc_mt),
            to_api_body_id(cx, rustc_body_id),
            to_api_syn_ty(cx, ty),
        )
    })
}
