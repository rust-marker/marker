use linter_api::ast::{
    ty::{
        ArrayTy, BoolTy, CommonTyData, EnumTy, FnTy, InferredTy, NeverTy, NumKind, NumTy, RawPtrTy, RefTy, SliceTy,
        StructTy, TextKind, TextTy, TraitObjTy, TupleTy, TyKind, UnionTy,
    },
    CallableData, Parameter,
};

use crate::{
    context::RustcContext,
    conversion::{to_api_abi, to_api_symbol_id},
};

use super::{
    generic::{to_api_generic_args, to_api_lifetime_from_syn, to_api_trait_bounds_from_hir},
    to_api_mutability, to_api_span_id, to_api_ty_def_id,
};

pub fn to_api_syn_ty<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    rustc_ty: &'tcx rustc_hir::Ty<'tcx>,
) -> TyKind<'ast> {
    let data = CommonTyData::new_syntactic(cx.ast_cx(), to_api_span_id(cx, rustc_ty.span));

    // Note about the usage of alloc. Here we can't reuse the types, as they
    // contain unique span. This might be avoidable with #43, but that might
    // not be perfect either. We can just keep it, until it becomes problematic.
    match &rustc_ty.kind {
        rustc_hir::TyKind::Slice(rustc_ty) => {
            TyKind::Slice(cx.storage.alloc(|| SliceTy::new(data, to_api_syn_ty(cx, rustc_ty))))
        },
        rustc_hir::TyKind::Array(rustc_ty, _) => {
            TyKind::Array(cx.storage.alloc(|| ArrayTy::new(data, to_api_syn_ty(cx, rustc_ty))))
        },
        rustc_hir::TyKind::Ptr(mut_ty) => TyKind::RawPtr(
            cx.storage
                .alloc(|| RawPtrTy::new(data, to_api_mutability(cx, mut_ty.mutbl), to_api_syn_ty(cx, mut_ty.ty))),
        ),
        rustc_hir::TyKind::Rptr(rust_lt, mut_ty) => TyKind::Ref(cx.storage.alloc(|| {
            RefTy::new(
                data,
                to_api_lifetime_from_syn(cx, rust_lt),
                to_api_mutability(cx, mut_ty.mutbl),
                to_api_syn_ty(cx, mut_ty.ty),
            )
        })),
        rustc_hir::TyKind::BareFn(rust_fn) => to_api_syn_ty_from_bare_fn(cx, data, rust_fn),
        rustc_hir::TyKind::Never => TyKind::Never(cx.storage.alloc(|| NeverTy::new(data))),
        rustc_hir::TyKind::Tup(rustc_tys) => {
            let api_tys = cx
                .storage
                .alloc_slice_iter(rustc_tys.iter().map(|rustc_ty| to_api_syn_ty(cx, rustc_ty)));
            TyKind::Tuple(cx.storage.alloc(|| TupleTy::new(data, api_tys)))
        },
        rustc_hir::TyKind::Path(qpath) => to_api_syn_ty_from_qpath(cx, data, qpath),
        rustc_hir::TyKind::OpaqueDef(_, _, _) => {
            // This requires function items to be implemented. Therefore we'll leave this as an open TODO for
            // now
            todo!("{:#?}", rustc_ty)
        },
        rustc_hir::TyKind::TraitObject(rust_bounds, rust_lt, _syntax) => TyKind::TraitObj(
            cx.storage
                .alloc(|| TraitObjTy::new(data, to_api_trait_bounds_from_hir(cx, rust_bounds, rust_lt))),
        ),
        rustc_hir::TyKind::Infer => TyKind::Inferred(cx.storage.alloc(|| InferredTy::new(data))),
        rustc_hir::TyKind::Err => unreachable!("would have triggered a rustc error"),
        rustc_hir::TyKind::Typeof(_) => unreachable!("docs state: 'Unused for now.'"),
    }
}

fn to_api_syn_ty_from_qpath<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    data: CommonTyData<'ast>,
    qpath: &rustc_hir::QPath<'tcx>,
) -> TyKind<'ast> {
    // FIXME: These `todo!()`s are currently not testable with the limited items
    // that are in the API. Therefore we'll leave them for another day.
    match qpath {
        rustc_hir::QPath::Resolved(None, path) => match path.res {
            rustc_hir::def::Res::Def(
                rustc_hir::def::DefKind::Enum | rustc_hir::def::DefKind::Struct | rustc_hir::def::DefKind::Union,
                id,
            ) => to_api_syn_ty_from_adt_def_id(cx, data, id, path.segments.last().and_then(|s| s.args)),
            rustc_hir::def::Res::Def(_, _) => todo!(),
            rustc_hir::def::Res::PrimTy(prim_ty) => to_api_syn_ty_from_prim_ty(cx, data, prim_ty),
            rustc_hir::def::Res::SelfTyParam { .. } => todo!(),
            rustc_hir::def::Res::SelfTyAlias { .. } => todo!(),
            rustc_hir::def::Res::SelfCtor(_) => todo!(),
            rustc_hir::def::Res::Local(_) => todo!(),
            rustc_hir::def::Res::ToolMod => todo!(),
            rustc_hir::def::Res::NonMacroAttr(_) => todo!(),
            rustc_hir::def::Res::Err => unreachable!("would have triggered a rustc error"),
        },
        rustc_hir::QPath::Resolved(_, _) => todo!(),
        rustc_hir::QPath::TypeRelative(_, _) => todo!(),
        rustc_hir::QPath::LangItem(_, _, _) => todo!(),
    }
}

fn to_api_syn_ty_from_adt_def_id<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    data: CommonTyData<'ast>,
    adt_id: rustc_hir::def_id::DefId,
    rustc_args: Option<&'tcx rustc_hir::GenericArgs<'tcx>>,
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
    prim_ty: rustc_hir::PrimTy,
) -> TyKind<'ast> {
    let num_kind = match prim_ty {
        rustc_hir::PrimTy::Int(int_ty) => match int_ty {
            rustc_ast::IntTy::Isize => NumKind::Isize,
            rustc_ast::IntTy::I8 => NumKind::I8,
            rustc_ast::IntTy::I16 => NumKind::I16,
            rustc_ast::IntTy::I32 => NumKind::I32,
            rustc_ast::IntTy::I64 => NumKind::I64,
            rustc_ast::IntTy::I128 => NumKind::I128,
        },
        rustc_hir::PrimTy::Uint(uint_ty) => match uint_ty {
            rustc_ast::UintTy::Usize => NumKind::Usize,
            rustc_ast::UintTy::U8 => NumKind::U8,
            rustc_ast::UintTy::U16 => NumKind::U16,
            rustc_ast::UintTy::U32 => NumKind::U32,
            rustc_ast::UintTy::U64 => NumKind::U64,
            rustc_ast::UintTy::U128 => NumKind::U128,
        },
        rustc_hir::PrimTy::Float(kind) => match kind {
            rustc_ast::FloatTy::F32 => NumKind::F32,
            rustc_ast::FloatTy::F64 => NumKind::F64,
        },
        rustc_hir::PrimTy::Str => return TyKind::Text(cx.storage.alloc(|| TextTy::new(data, TextKind::Str))),
        rustc_hir::PrimTy::Bool => return TyKind::Bool(cx.storage.alloc(|| BoolTy::new(data))),
        rustc_hir::PrimTy::Char => {
            return TyKind::Text(cx.storage.alloc(|| TextTy::new(data, TextKind::Char)));
        },
    };
    TyKind::Num(cx.storage.alloc(|| NumTy::new(data, num_kind)))
}

fn to_api_syn_ty_from_bare_fn<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    data: CommonTyData<'ast>,
    rust_fn: &rustc_hir::BareFnTy<'tcx>,
) -> TyKind<'ast> {
    assert_eq!(rust_fn.param_names.len(), rust_fn.decl.inputs.len());
    let params = rust_fn
        .decl
        .inputs
        .iter()
        .zip(rust_fn.param_names.iter())
        .map(|(rustc_ty, name)| {
            Parameter::new(
                cx.ast_cx(),
                Some(to_api_symbol_id(cx, name.name)),
                Some(to_api_syn_ty(cx, rustc_ty)),
                Some(to_api_span_id(cx, name.span)),
            )
        });
    let params = cx.storage.alloc_slice_iter(params);
    let return_ty = if let rustc_hir::FnRetTy::Return(rust_ty) = rust_fn.decl.output {
        Some(to_api_syn_ty(cx, rust_ty))
    } else {
        None
    };
    TyKind::Fn(cx.storage.alloc(|| {
        FnTy::new(
            data,
            CallableData::new(
                false,
                false,
                matches!(rust_fn.unsafety, rustc_hir::Unsafety::Unsafe),
                false,
                to_api_abi(cx, rust_fn.abi),
                false,
                params,
                return_ty,
            ),
        )
    }))
}
