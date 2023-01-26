use marker_api::ast::{
    ty::{
        AliasTy, ArrayTy, BoolTy, CommonTyData, EnumTy, FnTy, GenericTy, ImplTraitTy, InferredTy, NeverTy, NumKind,
        NumTy, RawPtrTy, RefTy, RelativeTy, SelfTy, SliceTy, StructTy, TextKind, TextTy, TraitObjTy, TupleTy, TyKind,
        UnionTy,
    },
    CommonCallableData, Parameter,
};
use rustc_hir as hir;

use super::MarkerConversionContext;

pub enum TySource<'tcx> {
    Syn(&'tcx hir::Ty<'tcx>),
}

impl<'tcx> From<&'tcx hir::Ty<'tcx>> for TySource<'tcx> {
    fn from(value: &'tcx hir::Ty<'tcx>) -> Self {
        TySource::Syn(value)
    }
}

impl<'ast, 'tcx> MarkerConversionContext<'ast, 'tcx> {
    #[must_use]
    pub fn to_ty(&self, source: impl Into<TySource<'tcx>>) -> TyKind<'ast> {
        let source: TySource<'tcx> = source.into();
        match source {
            TySource::Syn(syn_ty) => self.to_syn_ty(syn_ty),
        }
    }
}

impl<'ast, 'tcx> MarkerConversionContext<'ast, 'tcx> {
    #[must_use]
    fn to_syn_ty(&self, rustc_ty: &'tcx hir::Ty<'tcx>) -> TyKind<'ast> {
        let data = CommonTyData::new_syntactic(self.to_span_id(rustc_ty.span));

        // Note: Here we can't reuse allocated nodes, as each one contains
        // a unique span id. These nodes don't need to be stored individually, as
        // they can't be requested individually over the API. Instead, they're
        // always stored as part of a parent node.
        match &rustc_ty.kind {
            hir::TyKind::Slice(inner_ty) => TyKind::Slice(self.alloc(|| SliceTy::new(data, self.to_syn_ty(inner_ty)))),
            hir::TyKind::Array(inner_ty, _) => {
                TyKind::Array(self.alloc(|| ArrayTy::new(data, self.to_syn_ty(inner_ty))))
            },
            hir::TyKind::Ptr(mut_ty) => TyKind::RawPtr(self.alloc(|| {
                RawPtrTy::new(
                    data,
                    matches!(mut_ty.mutbl, rustc_ast::Mutability::Mut),
                    self.to_syn_ty(mut_ty.ty),
                )
            })),
            hir::TyKind::Ref(rust_lt, mut_ty) => TyKind::Ref(self.alloc(|| {
                RefTy::new(
                    data,
                    self.to_lifetime(rust_lt),
                    matches!(mut_ty.mutbl, rustc_ast::Mutability::Mut),
                    self.to_syn_ty(mut_ty.ty),
                )
            })),
            hir::TyKind::BareFn(rust_fn) => TyKind::Fn(self.alloc(|| self.to_syn_fn_ty(data, rust_fn))),
            hir::TyKind::Never => TyKind::Never(self.alloc(|| NeverTy::new(data))),
            hir::TyKind::Tup(rustc_tys) => {
                let api_tys = self.alloc_slice_iter(rustc_tys.iter().map(|rustc_ty| self.to_syn_ty(rustc_ty)));
                TyKind::Tuple(self.alloc(|| TupleTy::new(data, api_tys)))
            },
            hir::TyKind::Path(qpath) => self.to_syn_ty_from_qpath(data, qpath),
            // Continue ty conversion
            hir::TyKind::Err => unreachable!("would have triggered a rustc error"),
            hir::TyKind::Typeof(_) => unreachable!("docs state: 'Unused for now.'"),
            hir::TyKind::OpaqueDef(id, _, _) => {
                // `impl Trait` in rustc are implemented as Items with the kind `OpaqueTy`
                let item = self.rustc_cx.hir().item(*id);
                let hir::ItemKind::OpaqueTy(opty) = &item.kind else {
                    unreachable!("the item of a `OpaqueDef` should be `OpaqueTy` {item:#?}");
                };
                let rust_bound = self.to_ty_param_bound(opty.bounds);
                // FIXME: Generics are a bit weird with opaque types
                TyKind::ImplTrait(self.alloc(|| ImplTraitTy::new(data, rust_bound)))
            },
            hir::TyKind::TraitObject(rust_bounds, rust_lt, _syntax) => TyKind::TraitObj(
                self.alloc(|| TraitObjTy::new(data, self.to_ty_param_bound_from_hir(rust_bounds, rust_lt))),
            ),
            hir::TyKind::Infer => TyKind::Inferred(self.alloc(|| InferredTy::new(data))),
        }
    }

    #[must_use]
    pub fn to_syn_fn_ty(&self, data: CommonTyData<'ast>, rust_fn: &hir::BareFnTy<'tcx>) -> FnTy<'ast> {
        assert_eq!(rust_fn.param_names.len(), rust_fn.decl.inputs.len());
        let params = rust_fn
            .decl
            .inputs
            .iter()
            .zip(rust_fn.param_names.iter())
            .map(|(rustc_ty, name)| {
                Parameter::new(
                    Some(self.to_symbol_id(name.name)),
                    Some(self.to_syn_ty(rustc_ty)),
                    Some(self.to_span_id(name.span)),
                )
            });
        let params = self.alloc_slice_iter(params);
        let return_ty = if let hir::FnRetTy::Return(rust_ty) = rust_fn.decl.output {
            Some(self.to_syn_ty(rust_ty))
        } else {
            None
        };
        FnTy::new(
            data,
            CommonCallableData::new(
                false,
                false,
                matches!(rust_fn.unsafety, hir::Unsafety::Unsafe),
                false,
                self.to_abi(rust_fn.abi),
                false,
                params,
                return_ty,
            ),
        )
    }

    fn to_syn_ty_from_qpath(&self, data: CommonTyData<'ast>, qpath: &hir::QPath<'tcx>) -> TyKind<'ast> {
        match qpath {
            hir::QPath::Resolved(None, path) => match path.res {
                hir::def::Res::Def(
                    hir::def::DefKind::Enum | hir::def::DefKind::Struct | hir::def::DefKind::Union,
                    id,
                ) => self.to_syn_ty_from_adt_id(data, id, path.segments.last().and_then(|s| s.args)),
                hir::def::Res::Def(
                    hir::def::DefKind::LifetimeParam | hir::def::DefKind::TyParam | hir::def::DefKind::ConstParam,
                    id,
                ) => TyKind::Generic(self.alloc(|| GenericTy::new(data, self.to_generic_id(id)))),
                hir::def::Res::Def(hir::def::DefKind::TyAlias, id) => {
                    TyKind::Alias(self.alloc(|| AliasTy::new(data, self.to_item_id(id))))
                },
                hir::def::Res::PrimTy(prim_ty) => self.to_syn_ty_from_prim_ty(data, prim_ty),
                hir::def::Res::SelfTyParam { trait_: def_id, .. }
                | hir::def::Res::SelfTyAlias { alias_to: def_id, .. } => {
                    TyKind::SelfTy(self.alloc(|| SelfTy::new(data, self.to_item_id(def_id))))
                },
                hir::def::Res::Def(_, _)
                | hir::def::Res::SelfCtor(_)
                | hir::def::Res::Local(_)
                | hir::def::Res::ToolMod
                | hir::def::Res::NonMacroAttr(_) => unreachable!("not a syntactic type {path:#?}"),
                hir::def::Res::Err => unreachable!("would have triggered a rustc error"),
            },
            hir::QPath::Resolved(_, _) => todo!(),
            hir::QPath::TypeRelative(ty, segment) => TyKind::Relative(
                self.alloc(|| RelativeTy::new(data, self.to_syn_ty(ty), self.to_symbol_id(segment.ident.name))),
            ),
            hir::QPath::LangItem(_, _, _) => todo!(),
        }
    }

    fn to_syn_ty_from_adt_id(
        &self,
        data: CommonTyData<'ast>,
        adt_id: hir::def_id::DefId,
        rustc_args: Option<&'tcx hir::GenericArgs<'tcx>>,
    ) -> TyKind<'ast> {
        let adt = self.rustc_cx.adt_def(adt_id);
        let def_id = self.to_ty_def_id(adt_id);
        let generic_args = self.to_generic_args(rustc_args);
        match adt.adt_kind() {
            rustc_middle::ty::AdtKind::Struct => TyKind::Struct(
                self.alloc(|| StructTy::new(data, def_id, generic_args, adt.is_variant_list_non_exhaustive())),
            ),
            rustc_middle::ty::AdtKind::Enum => TyKind::Enum(
                self.alloc(|| EnumTy::new(data, def_id, generic_args, adt.is_variant_list_non_exhaustive())),
            ),
            rustc_middle::ty::AdtKind::Union => TyKind::Union(self.alloc(|| UnionTy::new(data, def_id, generic_args))),
        }
    }

    fn to_syn_ty_from_prim_ty(&self, data: CommonTyData<'ast>, prim_ty: hir::PrimTy) -> TyKind<'ast> {
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
            hir::PrimTy::Str => return TyKind::Text(self.alloc(|| TextTy::new(data, TextKind::Str))),
            hir::PrimTy::Bool => return TyKind::Bool(self.alloc(|| BoolTy::new(data))),
            hir::PrimTy::Char => {
                return TyKind::Text(self.alloc(|| TextTy::new(data, TextKind::Char)));
            },
        };
        TyKind::Num(self.alloc(|| NumTy::new(data, num_kind)))
    }
}
