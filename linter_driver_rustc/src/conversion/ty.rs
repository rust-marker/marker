use linter_api::ast::{
    ty::{
        ArrayTy, BoolTy, CommonTyData, EnumTy, FnTy, GenericTy, InferredTy, NeverTy, NumKind, NumTy, RawPtrTy, RefTy,
        SliceTy, StructTy, TextKind, TextTy, TraitObjTy, TupleTy, TyKind, UnionTy,
    },
    CommonCallableData, Parameter,
};

use crate::{
    context::RustcContext,
    conversion::{to_api_abi, to_api_symbol_id},
};

use super::{
    generic::{to_api_generic_args, to_api_lifetime, to_api_trait_bounds_from_hir},
    to_api_generic_id, to_api_mutability, to_api_span_id, to_api_ty_def_id,
};

use rustc_hir as hir;

pub fn to_api_syn_ty<'ast, 'tcx>(cx: &'ast RustcContext<'ast, 'tcx>, rustc_ty: &'tcx hir::Ty<'tcx>) -> TyKind<'ast> {
    let data = CommonTyData::new_syntactic(to_api_span_id(cx, rustc_ty.span));

    // Note about the usage of alloc. Here we can't reuse the types, as they
    // contain unique span. This might be avoidable with #43, but that might
    // not be perfect either. We can just keep it, until it becomes problematic.
    match &rustc_ty.kind {
        hir::TyKind::Slice(rustc_ty) => {
            TyKind::Slice(cx.storage.alloc(|| SliceTy::new(data, to_api_syn_ty(cx, rustc_ty))))
        },
        hir::TyKind::Array(rustc_ty, _) => {
            TyKind::Array(cx.storage.alloc(|| ArrayTy::new(data, to_api_syn_ty(cx, rustc_ty))))
        },
        hir::TyKind::Ptr(mut_ty) => TyKind::RawPtr(
            cx.storage
                .alloc(|| RawPtrTy::new(data, to_api_mutability(cx, mut_ty.mutbl), to_api_syn_ty(cx, mut_ty.ty))),
        ),
        hir::TyKind::Rptr(rust_lt, mut_ty) => TyKind::Ref(cx.storage.alloc(|| {
            RefTy::new(
                data,
                to_api_lifetime(cx, rust_lt),
                to_api_mutability(cx, mut_ty.mutbl),
                to_api_syn_ty(cx, mut_ty.ty),
            )
        })),
        hir::TyKind::BareFn(rust_fn) => to_api_syn_ty_from_bare_fn(cx, data, rust_fn),
        hir::TyKind::Never => TyKind::Never(cx.storage.alloc(|| NeverTy::new(data))),
        hir::TyKind::Tup(rustc_tys) => {
            let api_tys = cx
                .storage
                .alloc_slice_iter(rustc_tys.iter().map(|rustc_ty| to_api_syn_ty(cx, rustc_ty)));
            TyKind::Tuple(cx.storage.alloc(|| TupleTy::new(data, api_tys)))
        },
        hir::TyKind::Path(qpath) => to_api_syn_ty_from_qpath(cx, data, qpath),
        hir::TyKind::OpaqueDef(_, _, _) => {
            // This requires function items to be implemented. Therefore we'll leave this as an open TODO for
            // now
            todo!("{:#?}", rustc_ty)
        },
        hir::TyKind::TraitObject(rust_bounds, rust_lt, _syntax) => TyKind::TraitObj(
            cx.storage
                .alloc(|| TraitObjTy::new(data, to_api_trait_bounds_from_hir(cx, rust_bounds, rust_lt))),
        ),
        hir::TyKind::Infer => TyKind::Inferred(cx.storage.alloc(|| InferredTy::new(data))),
        hir::TyKind::Err => unreachable!("would have triggered a rustc error"),
        hir::TyKind::Typeof(_) => unreachable!("docs state: 'Unused for now.'"),
    }
}

fn to_api_syn_ty_from_qpath<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    data: CommonTyData<'ast>,
    qpath: &hir::QPath<'tcx>,
) -> TyKind<'ast> {
    // FIXME: These `todo!()`s are currently not testable with the limited items
    // that are in the API. Therefore we'll leave them for another day.
    match qpath {
        hir::QPath::Resolved(None, path) => match path.res {
            hir::def::Res::Def(hir::def::DefKind::Enum | hir::def::DefKind::Struct | hir::def::DefKind::Union, id) => {
                to_api_syn_ty_from_adt_def_id(cx, data, id, path.segments.last().and_then(|s| s.args))
            },
            hir::def::Res::Def(
                hir::def::DefKind::LifetimeParam | hir::def::DefKind::TyParam | hir::def::DefKind::ConstParam,
                id,
            ) => TyKind::Generic(cx.storage.alloc(|| GenericTy::new(data, to_api_generic_id(cx, id)))),
            hir::def::Res::Def(_, _) => todo!(),
            hir::def::Res::PrimTy(prim_ty) => to_api_syn_ty_from_prim_ty(cx, data, prim_ty),
            hir::def::Res::SelfTyParam { .. } => todo!(),
            hir::def::Res::SelfTyAlias { .. } => todo!(),
            hir::def::Res::SelfCtor(_) => todo!(),
            hir::def::Res::Local(_) => todo!(),
            hir::def::Res::ToolMod => todo!(),
            hir::def::Res::NonMacroAttr(_) => todo!(),
            hir::def::Res::Err => unreachable!("would have triggered a rustc error"),
        },
        hir::QPath::Resolved(_, _) => todo!(),
        hir::QPath::TypeRelative(_, _) => todo!(),
        hir::QPath::LangItem(_, _, _) => todo!(),
    }
}

fn to_api_syn_ty_from_adt_def_id<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    data: CommonTyData<'ast>,
    adt_id: hir::def_id::DefId,
    rustc_args: Option<&'tcx hir::GenericArgs<'tcx>>,
) -> TyKind<'ast> {
    let adt = cx.rustc_cx.adt_def(adt_id);
    // You were working on ADT parsing to then do all types in Option<...>. Next you need to
    // fill in the variants and generic args. The second one will be more work but also will be more
    // important
    let def_id = to_api_ty_def_id(cx, adt_id);
    let generic_args = to_api_generic_args(cx, rustc_args);
    match adt.adt_kind() {
        rustc_middle::ty::AdtKind::Struct => TyKind::Struct(
            cx.storage
                .alloc(|| StructTy::new(data, def_id, generic_args, adt.is_variant_list_non_exhaustive())),
        ),
        rustc_middle::ty::AdtKind::Enum => TyKind::Enum(
            cx.storage
                .alloc(|| EnumTy::new(data, def_id, generic_args, adt.is_variant_list_non_exhaustive())),
        ),
        rustc_middle::ty::AdtKind::Union => {
            TyKind::Union(cx.storage.alloc(|| UnionTy::new(data, def_id, generic_args)))
        },
    }
}

fn to_api_syn_ty_from_prim_ty<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    data: CommonTyData<'ast>,
    prim_ty: hir::PrimTy,
) -> TyKind<'ast> {
    let num_kind = match prim_ty {
        hir::PrimTy::Int(int_ty) => match int_ty {
            rustc_ast::IntTy::Isize => NumKind::Isize,
            rustc_ast::IntTy::I8 => NumKind::I8,
            rustc_ast::IntTy::I16 => NumKind::I16,
            rustc_ast::IntTy::I32 => NumKind::I32,
            rustc_ast::IntTy::I64 => NumKind::I64,
            rustc_ast::IntTy::I128 => NumKind::I128,
        },
        hir::PrimTy::Uint(uint_ty) => match uint_ty {
            rustc_ast::UintTy::Usize => NumKind::Usize,
            rustc_ast::UintTy::U8 => NumKind::U8,
            rustc_ast::UintTy::U16 => NumKind::U16,
            rustc_ast::UintTy::U32 => NumKind::U32,
            rustc_ast::UintTy::U64 => NumKind::U64,
            rustc_ast::UintTy::U128 => NumKind::U128,
        },
        hir::PrimTy::Float(kind) => match kind {
            rustc_ast::FloatTy::F32 => NumKind::F32,
            rustc_ast::FloatTy::F64 => NumKind::F64,
        },
        hir::PrimTy::Str => return TyKind::Text(cx.storage.alloc(|| TextTy::new(data, TextKind::Str))),
        hir::PrimTy::Bool => return TyKind::Bool(cx.storage.alloc(|| BoolTy::new(data))),
        hir::PrimTy::Char => {
            return TyKind::Text(cx.storage.alloc(|| TextTy::new(data, TextKind::Char)));
        },
    };
    TyKind::Num(cx.storage.alloc(|| NumTy::new(data, num_kind)))
}

fn to_api_syn_ty_from_bare_fn<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    data: CommonTyData<'ast>,
    rust_fn: &hir::BareFnTy<'tcx>,
) -> TyKind<'ast> {
    assert_eq!(rust_fn.param_names.len(), rust_fn.decl.inputs.len());
    let params = rust_fn
        .decl
        .inputs
        .iter()
        .zip(rust_fn.param_names.iter())
        .map(|(rustc_ty, name)| {
            Parameter::new(
                Some(to_api_symbol_id(name.name)),
                Some(to_api_syn_ty(cx, rustc_ty)),
                Some(to_api_span_id(cx, name.span)),
            )
        });
    let params = cx.storage.alloc_slice_iter(params);
    let return_ty = if let hir::FnRetTy::Return(rust_ty) = rust_fn.decl.output {
        Some(to_api_syn_ty(cx, rust_ty))
    } else {
        None
    };
    TyKind::Fn(cx.storage.alloc(|| {
        FnTy::new(
            data,
            CommonCallableData::new(
                false,
                false,
                matches!(rust_fn.unsafety, hir::Unsafety::Unsafe),
                false,
                to_api_abi(cx, rust_fn.abi),
                false,
                params,
                return_ty,
            ),
        )
    }))
}
