#![expect(unused)]

use linter_api::ast::{
    item::{ExternCrateItem, GenericParam, ItemData, ItemId, ItemType, VisibilityKind},
    CrateId, Symbol,
};

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

    fn get_vis(&self) -> VisibilityKind<'ast> {
        match self.item.vis.node {
            rustc_hir::VisibilityKind::Public => VisibilityKind::PubSelf,
            rustc_hir::VisibilityKind::Crate(..) => VisibilityKind::PubCrate,
            rustc_hir::VisibilityKind::Restricted { .. } => unimplemented!("VisibilityKind::PubPath"),
            rustc_hir::VisibilityKind::Inherited => VisibilityKind::PubSuper,
        }
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

fn from_rustc<'ast, 'tcx>(
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
            ItemType::UseDeclaration(cx.alloc_with(|| RustcUseDeclItem { cx, item, data }))
        },
        rustc_hir::ItemKind::Static(..) => todo!(),
        rustc_hir::ItemKind::Const(..) => todo!(),
        rustc_hir::ItemKind::Fn(..) => todo!(),
        rustc_hir::ItemKind::Macro(..) => todo!(),
        rustc_hir::ItemKind::ForeignMod { .. } => todo!(),
        rustc_hir::ItemKind::GlobalAsm(..) => todo!(),
        rustc_hir::ItemKind::TyAlias(..) => todo!(),
        rustc_hir::ItemKind::OpaqueTy(..) => todo!(),
        rustc_hir::ItemKind::Enum(..) => todo!(),
        rustc_hir::ItemKind::Struct(..) => todo!(),
        rustc_hir::ItemKind::Union(..) => todo!(),
        rustc_hir::ItemKind::Trait(..) => todo!(),
        rustc_hir::ItemKind::TraitAlias(..) => todo!(),
        rustc_hir::ItemKind::Impl(..) => todo!(),
    })
}

mod extern_crate_item;
mod mod_item;
mod use_decl;
