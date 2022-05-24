#![expect(unused)]

use linter_api::ast::{
    item::{CommonItemData, ExternCrateItem, GenericParam, ItemData, ItemId, ItemType, StaticItem, Visibility},
    CrateId, Symbol,
};

mod extern_crate_item;
mod mod_item;
mod use_decl;

use self::{
    extern_crate_item::RustcExternCrateItem,
    mod_item::RustcModItem,
    use_decl::{RustcUseDeclItem, RustcUseDeclItemData},
};
use super::rustc::RustcContext;
use crate::ast::ToApi;

use std::fmt::Debug;

impl<'ast, 'tcx> ToApi<'ast, 'tcx, ItemId> for rustc_hir::def_id::DefId {
    fn to_api(&self, cx: &'ast RustcContext<'ast, 'tcx>) -> ItemId {
        ItemId::new(self.krate.to_api(cx), self.index.as_u32())
    }
}

#[derive(Debug)]
pub struct RustcItem<'ast, 'tcx, T: Debug> {
    pub(crate) cx: &'ast RustcContext<'ast, 'tcx>,
    pub(crate) item: &'tcx rustc_hir::Item<'tcx>,
    pub(crate) data: T,
}

pub trait RustcItemData<'ast> {
    fn as_api_item(&'ast self) -> ItemType<'ast>;
}

impl<'ast, 'tcx, T: Debug> ItemData<'ast> for RustcItem<'ast, 'tcx, T>
where
    Self: RustcItemData<'ast>,
{
    fn get_id(&self) -> ItemId {
        let def_id = self.item.def_id.to_def_id();
        ItemId::new(def_id.krate.to_api(self.cx), def_id.index.as_u32())
    }

    fn get_span(&self) -> &'ast dyn linter_api::ast::Span<'ast> {
        self.cx.new_span(self.item.span)
    }

    fn get_vis(&self) -> &Visibility<'ast> {
        // match self.item.vis.node {
        //     rustc_hir::VisibilityKind::Public => VisibilityKind::PubSelf,
        //     rustc_hir::VisibilityKind::Crate(..) => VisibilityKind::PubCrate,
        //     rustc_hir::VisibilityKind::Restricted { .. } => unimplemented!("VisibilityKind::PubPath"),
        //     rustc_hir::VisibilityKind::Inherited => VisibilityKind::PubSuper,
        // }
        todo!()
    }

    fn get_name(&self) -> Option<Symbol> {
        (!self.item.ident.name.is_empty()).then(|| self.item.ident.name.to_api(self.cx))
    }

    fn as_item(&'ast self) -> ItemType<'ast> {
        self.as_api_item()
    }

    fn get_attrs(&self) {
        todo!()
    }
}

pub fn from_rustc<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    item: &'tcx rustc_hir::Item<'tcx>,
) -> Option<ItemType<'ast>> {
    Some(match item.kind {
        rustc_hir::ItemKind::Mod(..) => ItemType::Mod(cx.alloc_with(|| RustcModItem {
            cx,
            item,
            data: RustcModItem::data_from_rustc(cx, item),
        })),
        rustc_hir::ItemKind::ExternCrate(..) => ItemType::ExternCrate(cx.alloc_with(|| RustcExternCrateItem {
            cx,
            item,
            data: RustcExternCrateItem::data_from_rustc(cx, item),
        })),
        rustc_hir::ItemKind::Use(..) => {
            let data = RustcUseDeclItemData::data_from_rustc(cx, item)?;
            ItemType::UseDecl(cx.alloc_with(|| RustcUseDeclItem { cx, item, data }))
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
        rustc_item.vis.node.to_api(cx),
        (!rustc_item.ident.name.is_empty()).then(|| rustc_item.ident.name.to_api(cx)),
    )
}

impl<'ast, 'tcx> ToApi<'ast, 'tcx, Visibility<'ast>> for rustc_hir::Visibility<'tcx> {
    fn to_api(&self, cx: &'ast RustcContext<'ast, 'tcx>) -> Visibility<'ast> {
        self.node.to_api(cx)
    }
}

impl<'ast, 'tcx> ToApi<'ast, 'tcx, Visibility<'ast>> for rustc_hir::VisibilityKind<'tcx> {
    fn to_api(&self, cx: &'ast RustcContext<'ast, 'tcx>) -> Visibility<'ast> {
        match self {
            rustc_hir::VisibilityKind::Public => Visibility::PubSelf,
            rustc_hir::VisibilityKind::Crate(..) => Visibility::PubCrate,
            rustc_hir::VisibilityKind::Restricted { .. } => unimplemented!("VisibilityKind::PubPath"),
            rustc_hir::VisibilityKind::Inherited => Visibility::PubSuper,
        }
    }
}
