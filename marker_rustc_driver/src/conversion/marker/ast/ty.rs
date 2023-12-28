use marker_api::{
    ast::{
        ArrayTy, BoolTy, CommonSynTyData, FnPtrTy, FnTyParameter, ImplTraitTy, InferredTy, NeverTy, NumTy, PathTy,
        RawPtrTy, RefTy, SliceTy, TextTy, TraitObjTy, TupleTy, TyKind,
    },
    common::{NumKind, TextKind},
};
use rustc_hir as hir;

use crate::conversion::marker::MarkerConverterInner;

impl<'ast, 'tcx> MarkerConverterInner<'ast, 'tcx> {
    #[must_use]
    pub fn to_syn_ty(&self, rustc_ty: &'tcx hir::Ty<'tcx>) -> TyKind<'ast> {
        let data = CommonSynTyData::new_syntactic(self.to_span_id(rustc_ty.span));

        // Note: Here we can't reuse allocated nodes, as each one contains
        // a unique span id. These nodes don't need to be stored individually, as
        // they can't be requested individually over the API. Instead, they're
        // always stored as part of a parent node.
        match &rustc_ty.kind {
            hir::TyKind::Slice(inner_ty) => TyKind::Slice(self.alloc(SliceTy::new(data, self.to_syn_ty(inner_ty)))),
            hir::TyKind::Array(inner_ty, rust_len) => {
                let len = match rust_len {
                    hir::ArrayLen::Body(anon) => Some(self.to_const_expr(*anon)),
                    hir::ArrayLen::Infer(_, _) => None,
                };
                TyKind::Array(self.alloc(ArrayTy::new(data, self.to_syn_ty(inner_ty), len)))
            },
            hir::TyKind::Ptr(mut_ty) => TyKind::RawPtr(self.alloc(RawPtrTy::new(
                data,
                self.to_mutability(mut_ty.mutbl),
                self.to_syn_ty(mut_ty.ty),
            ))),
            hir::TyKind::Ref(rust_lt, mut_ty) => TyKind::Ref(self.alloc({
                RefTy::new(
                    data,
                    self.to_lifetime(rust_lt),
                    self.to_mutability(mut_ty.mutbl),
                    self.to_syn_ty(mut_ty.ty),
                )
            })),
            hir::TyKind::BareFn(rust_fn) => TyKind::FnPtr(self.alloc(self.to_syn_fn_prt_ty(data, rust_fn))),
            hir::TyKind::Never => TyKind::Never(self.alloc(NeverTy::new(data))),
            hir::TyKind::Tup(rustc_tys) => {
                let api_tys = self.alloc_slice(rustc_tys.iter().map(|rustc_ty| self.to_syn_ty(rustc_ty)));
                TyKind::Tuple(self.alloc(TupleTy::new(data, api_tys)))
            },
            hir::TyKind::Path(qpath) => self.to_syn_ty_from_qpath(data, qpath, rustc_ty),
            // Continue ty conversion
            hir::TyKind::Err(..) => unreachable!("would have triggered a rustc error"),
            hir::TyKind::Typeof(_) => unreachable!("docs state: 'Unused for now.'"),
            hir::TyKind::OpaqueDef(id, _, _) => {
                // `impl Trait` in rustc are implemented as Items with the kind `OpaqueTy`
                let item = self.rustc_cx.hir().item(*id);
                let hir::ItemKind::OpaqueTy(opty) = &item.kind else {
                    unreachable!("the item of a `OpaqueDef` should be `OpaqueTy` {item:#?}");
                };
                let rust_bound = self.to_syn_ty_param_bound(opty.bounds);
                // FIXME: Generics are a bit weird with opaque types
                TyKind::ImplTrait(self.alloc(ImplTraitTy::new(data, rust_bound)))
            },
            hir::TyKind::TraitObject(rust_bounds, rust_lt, _syntax) => TyKind::TraitObj(self.alloc(TraitObjTy::new(
                data,
                self.to_syn_ty_param_bound_from_hir(rust_bounds, rust_lt),
            ))),
            hir::TyKind::Infer => TyKind::Inferred(self.alloc(InferredTy::new(data))),
        }
    }

    #[must_use]
    pub fn to_syn_fn_prt_ty(&self, data: CommonSynTyData<'ast>, rust_fn: &hir::BareFnTy<'tcx>) -> FnPtrTy<'ast> {
        assert_eq!(rust_fn.param_names.len(), rust_fn.decl.inputs.len());
        let params = rust_fn
            .decl
            .inputs
            .iter()
            .zip(rust_fn.param_names.iter())
            .map(|(rustc_ty, name)| {
                let (param_span, ident) = if name.span.is_dummy() {
                    (rustc_ty.span, None)
                } else {
                    let span = name.span.until(rustc_ty.span);
                    (span, Some(self.to_ident(*name)))
                };
                FnTyParameter::builder()
                    .ident(ident)
                    .span(self.to_span_id(param_span))
                    .ty(self.to_syn_ty(rustc_ty))
                    .build()
            });
        let params = self.alloc_slice(params);
        let return_ty = if let hir::FnRetTy::Return(rust_ty) = rust_fn.decl.output {
            Some(self.to_syn_ty(rust_ty))
        } else {
            None
        };
        FnPtrTy::builder()
            .data(data)
            .safety(self.to_safety(rust_fn.unsafety))
            .abi(self.to_abi(rust_fn.abi))
            .params(params)
            .return_ty(return_ty)
            .build()
    }

    fn to_syn_ty_from_qpath(
        &self,
        data: CommonSynTyData<'ast>,
        qpath: &hir::QPath<'tcx>,
        rustc_ty: &hir::Ty<'_>,
    ) -> TyKind<'ast> {
        match qpath {
            hir::QPath::Resolved(_, path) => match path.res {
                hir::def::Res::Def(
                    hir::def::DefKind::LifetimeParam
                    | hir::def::DefKind::TyParam
                    | hir::def::DefKind::ConstParam
                    | hir::def::DefKind::TyAlias { .. }
                    | hir::def::DefKind::Enum
                    | hir::def::DefKind::Struct
                    | hir::def::DefKind::Union
                    | hir::def::DefKind::Trait
                    | hir::def::DefKind::AssocTy
                    | hir::def::DefKind::ForeignTy
                    | hir::def::DefKind::TraitAlias,
                    _,
                )
                | hir::def::Res::SelfTyParam { .. }
                | hir::def::Res::SelfTyAlias { .. } => {
                    TyKind::Path(self.alloc(PathTy::new(data, self.to_qpath_from_ty(qpath, rustc_ty))))
                },
                hir::def::Res::PrimTy(prim_ty) => self.to_syn_ty_from_prim_ty(data, prim_ty),
                hir::def::Res::Def(_, _)
                | hir::def::Res::SelfCtor(_)
                | hir::def::Res::Local(_)
                | hir::def::Res::ToolMod
                | hir::def::Res::NonMacroAttr(_) => unreachable!("not a syntactic type {path:#?}"),
                hir::def::Res::Err => unreachable!("would have triggered a rustc error"),
            },
            hir::QPath::TypeRelative(_, _) | hir::QPath::LangItem(_, _) => {
                TyKind::Path(self.alloc(PathTy::new(data, self.to_qpath_from_ty(qpath, rustc_ty))))
            },
        }
    }

    fn to_syn_ty_from_prim_ty(&self, data: CommonSynTyData<'ast>, prim_ty: hir::PrimTy) -> TyKind<'ast> {
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
            hir::PrimTy::Str => return TyKind::Text(self.alloc(TextTy::new(data, TextKind::Str))),
            hir::PrimTy::Bool => return TyKind::Bool(self.alloc(BoolTy::new(data))),
            hir::PrimTy::Char => {
                return TyKind::Text(self.alloc(TextTy::new(data, TextKind::Char)));
            },
        };
        TyKind::Num(self.alloc(NumTy::new(data, num_kind)))
    }
}
