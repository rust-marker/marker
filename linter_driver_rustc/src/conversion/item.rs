use std::cell::RefCell;

use linter_api::ast::{
    generic::{
        GenericParamKind, GenericParams, LifetimeClause, LifetimeParam, TraitBound, TyClause, TyParam, TyParamBound,
        WhereClauseKind,
    },
    item::{CommonItemData, ConstItem, ExternCrateItem, ItemKind, ModItem, StaticItem, TyAliasItem},
    ty::TyKind,
    ItemId,
};
use rustc_hash::FxHashMap;
use rustc_hir as hir;

use crate::context::RustcContext;

use super::{
    generic::{to_api_lifetime, to_api_trait_ref},
    to_api_body_id, to_api_generic_id, to_api_item_id_from_def_id, to_api_mutability, to_api_span_id, to_api_symbol_id,
    ty::to_api_syn_ty,
};

pub struct ItemConverter<'ast, 'tcx> {
    cx: &'ast RustcContext<'ast, 'tcx>,
    items: RefCell<FxHashMap<ItemId, ItemKind<'ast>>>,
}

impl<'ast, 'tcx> ItemConverter<'ast, 'tcx> {
    pub fn new(cx: &'ast RustcContext<'ast, 'tcx>) -> Self {
        Self {
            cx,
            items: RefCell::default(),
        }
    }

    pub fn conv_item(&self, rustc_item: &hir::Item<'tcx>) -> Option<ItemKind<'ast>> {
        let id = to_api_item_id_from_def_id(self.cx, rustc_item.owner_id.to_def_id());
        if let Some(item) = self.items.borrow().get(&id) {
            return Some(*item);
        }

        let name = to_api_symbol_id(self.cx, rustc_item.ident.name);
        let data = CommonItemData::new(id, name);
        let item = match rustc_item.kind {
            hir::ItemKind::ExternCrate(original_name) => ItemKind::ExternCrate(self.alloc(|| {
                ExternCrateItem::new(data, original_name.map_or(name, |sym| to_api_symbol_id(self.cx, sym)))
            })),
            hir::ItemKind::Use(_, _) => todo!(),
            hir::ItemKind::Static(rustc_ty, rustc_mut, rustc_body_id) => ItemKind::Static(self.alloc(|| {
                StaticItem::new(
                    data,
                    to_api_mutability(self.cx, rustc_mut),
                    to_api_body_id(self.cx, rustc_body_id),
                    self.conv_ty(rustc_ty),
                )
            })),
            hir::ItemKind::Const(rustc_ty, rustc_body_id) => ItemKind::Const(self.alloc(|| {
                ConstItem::new(
                    data,
                    self.conv_ty(rustc_ty),
                    Some(to_api_body_id(self.cx, rustc_body_id)),
                )
            })),
            hir::ItemKind::Fn(_, _, _) => todo!(),
            hir::ItemKind::Macro(_, _) => return None,
            hir::ItemKind::Mod(rustc_mod) => {
                ItemKind::Mod(self.alloc(|| ModItem::new(data, self.conv_item_slice(rustc_mod.item_ids))))
            },
            hir::ItemKind::ForeignMod { .. } => todo!(),
            hir::ItemKind::GlobalAsm(_) => return None,
            hir::ItemKind::TyAlias(rustc_ty, rustc_generics) => ItemKind::TyAlias(
                self.alloc(|| TyAliasItem::new(data, self.conv_generic(rustc_generics), Some(self.conv_ty(rustc_ty)))),
            ),
            hir::ItemKind::OpaqueTy(_) => todo!(),
            hir::ItemKind::Enum(_, _) => todo!(),
            hir::ItemKind::Struct(_, _) => todo!(),
            hir::ItemKind::Union(_, _) => todo!(),
            hir::ItemKind::Trait(_, _, _, _, _) => todo!(),
            hir::ItemKind::TraitAlias(_, _) => todo!(),
            hir::ItemKind::Impl(_) => todo!(),
        };

        self.items.borrow_mut().insert(id, item);
        Some(item)
    }

    fn conv_item_slice(&self, item: &[hir::ItemId]) -> &'ast [ItemKind<'ast>] {
        #[expect(
            clippy::needless_collect,
            reason = "collect is required to know the size of the allocation"
        )]
        let items: Vec<ItemKind<'_>> = item
            .iter()
            .map(|rid| self.cx.rustc_cx.hir().item(*rid))
            .filter_map(|rustc_item| self.conv_item(rustc_item))
            .collect();
        self.cx.storage.alloc_slice_iter(items.into_iter())
    }

    fn conv_ty(&self, rustc_ty: &'tcx hir::Ty<'tcx>) -> TyKind<'ast> {
        to_api_syn_ty(self.cx, rustc_ty)
    }

    fn conv_generic_params(&self, params: &[hir::GenericParam<'ast>]) -> &'ast [GenericParamKind<'ast>] {
        if params.is_empty() {
            return &[];
        }

        let params: Vec<_> = params
            .iter()
            .filter_map(|rustc_param| {
                let name = match rustc_param.name {
                    hir::ParamName::Plain(ident) => to_api_symbol_id(self.cx, ident.name),
                    _ => return None,
                };
                let id = to_api_generic_id(self.cx, rustc_param.hir_id.expect_owner().to_def_id());
                let span = to_api_span_id(self.cx, rustc_param.span);
                match rustc_param.kind {
                    hir::GenericParamKind::Lifetime {
                        kind: hir::LifetimeParamKind::Explicit,
                    } => Some(GenericParamKind::Lifetime(
                        self.alloc(|| LifetimeParam::new(id, name, &[], Some(span))),
                    )),
                    hir::GenericParamKind::Type { synthetic: false, .. } => Some(GenericParamKind::Ty(
                        self.alloc(|| TyParam::new(Some(span), name, id, &[])),
                    )),
                    _ => None,
                }
            })
            .collect();

        self.cx.storage.alloc_slice_iter(params.into_iter())
    }

    fn conv_generic_bounds(&self, bounds: hir::GenericBounds<'tcx>) -> &'ast [TyParamBound<'ast>] {
        if bounds.is_empty() {
            return &[];
        }

        let bounds: Vec<_> = bounds
            .iter()
            .filter_map(|bound| match bound {
                hir::GenericBound::Trait(trait_ref, modifier) => Some(TyParamBound::TraitBound(self.alloc(|| {
                    TraitBound::new(
                        !matches!(modifier, hir::TraitBoundModifier::None),
                        to_api_trait_ref(self.cx, trait_ref),
                        to_api_span_id(self.cx, bound.span()),
                    )
                }))),
                hir::GenericBound::LangItemTrait(_, _, _, _) => todo!(),
                hir::GenericBound::Outlives(rust_lt) => {
                    to_api_lifetime(self.cx, rust_lt).map(|api_lt| TyParamBound::Lifetime(self.alloc(|| api_lt)))
                },
            })
            .collect();

        self.cx.storage.alloc_slice_iter(bounds.into_iter())
    }

    fn conv_generic(&self, rustc_generics: &hir::Generics<'tcx>) -> GenericParams<'ast> {
        // FIXME: Update implementation to store the parameter bounds in the parameters
        let clauses: Vec<_> = rustc_generics
            .predicates
            .iter()
            .filter_map(|predicate| {
                match predicate {
                    hir::WherePredicate::BoundPredicate(ty_bound) => {
                        // FIXME Add span to API clause:
                        // let span = to_api_span_id(self.cx, ty_bound.span);
                        let params = GenericParams::new(self.conv_generic_params(ty_bound.bound_generic_params), &[]);
                        let ty = self.conv_ty(ty_bound.bounded_ty);
                        Some(WhereClauseKind::Ty(self.alloc(|| {
                            TyClause::new(Some(params), ty, self.conv_generic_bounds(predicate.bounds()))
                        })))
                    },
                    hir::WherePredicate::RegionPredicate(lifetime_bound) => {
                        to_api_lifetime(self.cx, lifetime_bound.lifetime).map(|lifetime| {
                            WhereClauseKind::Lifetime(self.alloc(|| {
                                let bounds: Vec<_> = lifetime_bound
                                    .bounds
                                    .iter()
                                    .filter_map(|bound| match bound {
                                        hir::GenericBound::Outlives(lifetime) => to_api_lifetime(self.cx, lifetime),
                                        _ => unreachable!("lifetimes can only be bound by lifetimes"),
                                    })
                                    .collect();
                                let bounds = if bounds.is_empty() {
                                    self.cx.storage.alloc_slice_iter(bounds.into_iter())
                                } else {
                                    &[]
                                };
                                LifetimeClause::new(lifetime, bounds)
                            }))
                        })
                    },
                    hir::WherePredicate::EqPredicate(_) => {
                        unreachable!("the documentation states, that this is unsupported")
                    },
                }
            })
            .collect();
        let clauses = self.cx.storage.alloc_slice_iter(clauses.into_iter());

        GenericParams::new(self.conv_generic_params(rustc_generics.params), clauses)
    }

    #[must_use]
    fn alloc<F, T>(&self, f: F) -> &'ast T
    where
        F: FnOnce() -> T,
    {
        self.cx.storage.alloc(f)
    }
}

pub fn to_api_item<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    rustc_item: &'tcx hir::Item<'tcx>,
) -> Option<ItemKind<'ast>> {
    let id = to_api_item_id_from_def_id(cx, rustc_item.owner_id.to_def_id());
    if let Some(item) = cx.storage.item(id) {
        return Some(item);
    }

    let common_data = CommonItemData::new(id, to_api_symbol_id(cx, rustc_item.ident.name));
    let item = match rustc_item.kind {
        hir::ItemKind::Mod(rustc_mod) => ItemKind::Mod(to_mod_item(cx, common_data, rustc_mod)),
        hir::ItemKind::Static(ty, mt, rust_body_id) => {
            ItemKind::Static(to_static_item(cx, common_data, ty, mt, rust_body_id))
        },
        hir::ItemKind::ExternCrate(_)
        | hir::ItemKind::Use(_, _)
        | hir::ItemKind::Const(_, _)
        | hir::ItemKind::Fn(_, _, _)
        | hir::ItemKind::Macro(_, _)
        | hir::ItemKind::ForeignMod { .. }
        | hir::ItemKind::GlobalAsm(_)
        | hir::ItemKind::TyAlias(_, _)
        | hir::ItemKind::OpaqueTy(_)
        | hir::ItemKind::Enum(_, _)
        | hir::ItemKind::Struct(_, _)
        | hir::ItemKind::Union(_, _)
        | hir::ItemKind::Trait(_, _, _, _, _)
        | hir::ItemKind::TraitAlias(_, _)
        | hir::ItemKind::Impl(_) => None?,
    };

    cx.storage.add_item(id, item);
    Some(item)
}

fn to_mod_item<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    data: CommonItemData<'ast>,
    rustc_mod: &hir::Mod,
) -> &'ast ModItem<'ast> {
    #[expect(
        clippy::needless_collect,
        reason = "collect is required to know the size of the allocation"
    )]
    let items: Vec<ItemKind<'_>> = rustc_mod
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
    ty: &'tcx hir::Ty<'tcx>,
    rustc_mt: rustc_ast::Mutability,
    rustc_body_id: hir::BodyId,
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
