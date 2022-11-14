use linter_api::ast::{
    generic::{BindingGenericArg, GenericArgKind, GenericArgs, Lifetime, LifetimeKind, TraitBound, TyParamBound},
    TraitRef,
};

use crate::context::RustcContext;

use super::{to_api_generic_id, to_api_item_id_from_def_id, to_api_span_id, to_api_symbol_id, ty::to_api_syn_ty};

pub fn to_api_lifetime<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    rust_lt: &rustc_hir::Lifetime,
) -> Option<Lifetime<'ast>> {
    let kind = match rust_lt.name {
        rustc_hir::LifetimeName::Param(local_id, rustc_hir::ParamName::Plain(ident)) => LifetimeKind::Label(
            to_api_symbol_id(ident.name),
            to_api_generic_id(cx, local_id.to_def_id()),
        ),
        rustc_hir::LifetimeName::Param(_local_id, rustc_hir::ParamName::Fresh) => todo!("{:#?}", rust_lt),
        rustc_hir::LifetimeName::ImplicitObjectLifetimeDefault => return None,
        rustc_hir::LifetimeName::Infer => LifetimeKind::Infer,
        rustc_hir::LifetimeName::Static => LifetimeKind::Static,
        rustc_hir::LifetimeName::Param(_, rustc_hir::ParamName::Error) | rustc_hir::LifetimeName::Error => {
            unreachable!("would have triggered a rustc error")
        },
    };

    Some(Lifetime::new(Some(to_api_span_id(cx, rust_lt.span)), kind))
}

pub fn to_api_generic_args_from_path<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    rust_path: &rustc_hir::Path<'tcx>,
) -> GenericArgs<'ast> {
    to_api_generic_args(cx, rust_path.segments.last().and_then(|s| s.args))
}

pub fn to_api_generic_args<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    rustc_args: Option<&'tcx rustc_hir::GenericArgs<'tcx>>,
) -> GenericArgs<'ast> {
    to_api_generic_args_opt(cx, rustc_args).unwrap_or_else(|| GenericArgs::new(&[]))
}

pub fn to_api_generic_args_opt<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    rustc_args: Option<&'tcx rustc_hir::GenericArgs<'tcx>>,
) -> Option<GenericArgs<'ast>> {
    let Some(rustc_args) = rustc_args else {
        return None;
    };

    let mut args: Vec<_> = rustc_args
        .args
        .iter()
        .filter(|rustc_arg| !rustc_arg.is_synthetic())
        .map(|rustc_arg| match rustc_arg {
            rustc_hir::GenericArg::Lifetime(r_lt) => {
                GenericArgKind::Lifetime(cx.storage.alloc(|| to_api_lifetime(cx, r_lt).unwrap()))
            },
            rustc_hir::GenericArg::Type(r_ty) => GenericArgKind::Ty(cx.storage.alloc(|| to_api_syn_ty(cx, r_ty))),
            rustc_hir::GenericArg::Const(_) => todo!(),
            rustc_hir::GenericArg::Infer(_) => todo!(),
        })
        .collect();
    args.extend(rustc_args.bindings.iter().map(|binding| match &binding.kind {
        rustc_hir::TypeBindingKind::Equality { term } => match term {
            rustc_hir::Term::Ty(rustc_ty) => GenericArgKind::Binding(cx.storage.alloc(|| {
                BindingGenericArg::new(
                    Some(to_api_span_id(cx, binding.span)),
                    to_api_symbol_id(binding.ident.name),
                    to_api_syn_ty(cx, rustc_ty),
                )
            })),
            rustc_hir::Term::Const(_) => todo!(),
        },
        rustc_hir::TypeBindingKind::Constraint { .. } => todo!(),
    }));
    Some(GenericArgs::new(cx.storage.alloc_slice_iter(args.drain(..))))
}

pub fn to_api_trait_ref<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    trait_ref: &rustc_hir::PolyTraitRef<'tcx>,
) -> TraitRef<'ast> {
    let trait_id = match trait_ref.trait_ref.path.res {
        rustc_hir::def::Res::Def(rustc_hir::def::DefKind::Trait | rustc_hir::def::DefKind::TraitAlias, rustc_id) => {
            to_api_item_id_from_def_id(cx, rustc_id)
        },
        _ => unreachable!("reached `PolyTraitRef` which can't be translated {trait_ref:#?}"),
    };
    // TODO get generic args from last path segment
    TraitRef::new(trait_id, to_api_generic_args_from_path(cx, trait_ref.trait_ref.path))
}

pub fn to_api_trait_bounds_from_hir<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    rust_bounds: &[rustc_hir::PolyTraitRef<'tcx>],
    rust_lt: &rustc_hir::Lifetime,
) -> &'ast [TyParamBound<'ast>] {
    let traits = rust_bounds.iter().map(|rust_trait_ref| {
        TyParamBound::TraitBound(cx.storage.alloc(|| {
            TraitBound::new(
                false,
                to_api_trait_ref(cx, rust_trait_ref),
                to_api_span_id(cx, rust_trait_ref.span),
            )
        }))
    });

    if let Some(lt) = to_api_lifetime(cx, rust_lt) {
        // alloc_slice_iter requires a const size, which is not possible otherwise
        let mut bounds: Vec<_> = traits.collect();
        bounds.push(TyParamBound::Lifetime(cx.storage.alloc(move || lt)));
        cx.storage.alloc_slice_iter(bounds.drain(..))
    } else {
        cx.storage.alloc_slice_iter(traits)
    }
}
