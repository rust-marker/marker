use linter_api::ast::ty::{BoolTy, CommonTyData, InferredTy, NumKind, NumTy, TextKind, TextTy, TyKind};

use crate::context::RustcContext;

use super::to_api_span_id;

pub fn to_api_syntactic_type<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    rustc_ty: &'tcx rustc_hir::Ty<'tcx>,
) -> TyKind<'ast> {
    let common_data = CommonTyData::new_syntactic(cx.ast_cx(), to_api_span_id(cx, rustc_ty.span));

    // Note about the usage of alloc. Here we can't reuse the types, as they
    // contain unique span. This might be avoidable with #43, but that might
    // not be perfect either. We can just keep it, until it becomes problematic.
    match &rustc_ty.kind {
        rustc_hir::TyKind::Slice(_)
        | rustc_hir::TyKind::Array(_, _)
        | rustc_hir::TyKind::Ptr(_)
        | rustc_hir::TyKind::Rptr(_, _)
        | rustc_hir::TyKind::BareFn(_)
        | rustc_hir::TyKind::Never
        | rustc_hir::TyKind::Tup(_) => panic!("{:#?}", rustc_ty),
        rustc_hir::TyKind::Path(qpath) => to_api_syn_ty_from_qpath(cx, common_data, qpath),
        rustc_hir::TyKind::OpaqueDef(_, _, _)
        | rustc_hir::TyKind::TraitObject(_, _, _)
        | rustc_hir::TyKind::Typeof(_) => panic!("{:#?}", rustc_ty),
        rustc_hir::TyKind::Infer => TyKind::Inferred(cx.storage.alloc(|| InferredTy::new(common_data))),
        rustc_hir::TyKind::Err => unreachable!("would have triggered a rustc error"),
    }
}

fn to_api_syn_ty_from_qpath<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    common_data: CommonTyData<'ast>,
    qpath: &rustc_hir::QPath<'tcx>,
) -> TyKind<'ast> {
    match qpath {
        rustc_hir::QPath::Resolved(None, path) => match path.res {
            rustc_hir::def::Res::Def(_, _) => todo!(),
            rustc_hir::def::Res::PrimTy(prim_ty) => to_api_syn_ty_from_prim_ty(cx, common_data, prim_ty),
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

fn to_api_syn_ty_from_prim_ty<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    common_data: CommonTyData<'ast>,
    prim_ty: rustc_hir::PrimTy,
) -> TyKind<'ast> {
    let num_kind;
    match prim_ty {
        rustc_hir::PrimTy::Int(int_ty) => {
            num_kind = match int_ty {
                rustc_ast::IntTy::Isize => NumKind::Isize,
                rustc_ast::IntTy::I8 => NumKind::I8,
                rustc_ast::IntTy::I16 => NumKind::I16,
                rustc_ast::IntTy::I32 => NumKind::I32,
                rustc_ast::IntTy::I64 => NumKind::I64,
                rustc_ast::IntTy::I128 => NumKind::I128,
            };
        },
        rustc_hir::PrimTy::Uint(uint_ty) => {
            num_kind = match uint_ty {
                rustc_ast::UintTy::Usize => NumKind::Usize,
                rustc_ast::UintTy::U8 => NumKind::U8,
                rustc_ast::UintTy::U16 => NumKind::U16,
                rustc_ast::UintTy::U32 => NumKind::U32,
                rustc_ast::UintTy::U64 => NumKind::U64,
                rustc_ast::UintTy::U128 => NumKind::U128,
            };
        },
        rustc_hir::PrimTy::Float(kind) => {
            num_kind = match kind {
                rustc_ast::FloatTy::F32 => NumKind::F32,
                rustc_ast::FloatTy::F64 => NumKind::F64,
            }
        },
        rustc_hir::PrimTy::Str => return TyKind::Text(cx.storage.alloc(|| TextTy::new(common_data, TextKind::Str))),
        rustc_hir::PrimTy::Bool => return TyKind::Bool(cx.storage.alloc(|| BoolTy::new(common_data))),
        rustc_hir::PrimTy::Char => {
            return TyKind::Text(cx.storage.alloc(|| TextTy::new(common_data, TextKind::Char)));
        },
    }
    TyKind::Num(cx.storage.alloc(|| NumTy::new(common_data, num_kind)))
}
