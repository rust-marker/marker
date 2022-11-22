use linter_api::ast::{
    ty::{
        AliasTy, ArrayTy, BoolTy, CommonTyData, EnumTy, FnTy, GenericTy, InferredTy, NeverTy, NumKind, NumTy, RawPtrTy,
        RefTy, RelativeTy, SelfTy, SliceTy, StructTy, TextKind, TextTy, TraitObjTy, TupleTy, TyKind, UnionTy,
    },
    CommonCallableData, Parameter,
};

use crate::{
    context::RustcContext,
    conversion::{to_api_abi, to_symbol_id},
};

use super::{
    generic::{to_api_generic_args, to_api_lifetime, to_api_trait_bounds_from_hir},
    to_api_mutability, to_generic_id, to_item_id, to_span_id, to_ty_def_id,
};

use rustc_hir as hir;

pub struct TyConverter<'ast, 'tcx> {
    cx: &'ast RustcContext<'ast, 'tcx>,
}

pub enum TySource<'tcx> {
    Syn(&'tcx hir::Ty<'tcx>),
}

impl<'tcx> From<&'tcx hir::Ty<'tcx>> for TySource<'tcx> {
    fn from(value: &'tcx hir::Ty<'tcx>) -> Self {
        TySource::Syn(value)
    }
}

impl<'ast, 'tcx> TyConverter<'ast, 'tcx> {
    pub fn new(cx: &'ast RustcContext<'ast, 'tcx>) -> Self {
        Self { cx }
    }

    pub fn conv_ty(&self, source: impl Into<TySource<'tcx>>) -> TyKind<'ast> {
        let source: TySource<'tcx> = source.into();
        match source {
            TySource::Syn(syn_ty) => self.conv_syn_ty(syn_ty),
        }
    }

    fn conv_syn_ty(&self, rustc_ty: &'tcx hir::Ty<'tcx>) -> TyKind<'ast> {
        let data = CommonTyData::new_syntactic(to_span_id(rustc_ty.span));

        // Note about the usage of alloc. Here we can't reuse the types, as they
        // contain unique span. This might be avoidable with #43, but that might
        // not be perfect either. We can just keep it, until it becomes problematic.
        match &rustc_ty.kind {
            hir::TyKind::Slice(rustc_ty) => {
                TyKind::Slice(self.cx.storage.alloc(|| SliceTy::new(data, self.conv_syn_ty(rustc_ty))))
            },
            hir::TyKind::Array(rustc_ty, _) => {
                TyKind::Array(self.cx.storage.alloc(|| ArrayTy::new(data, self.conv_syn_ty(rustc_ty))))
            },
            hir::TyKind::Ptr(mut_ty) => TyKind::RawPtr(
                self.cx
                    .storage
                    .alloc(|| RawPtrTy::new(data, to_api_mutability(mut_ty.mutbl), self.conv_syn_ty(mut_ty.ty))),
            ),
            hir::TyKind::Rptr(rust_lt, mut_ty) => TyKind::Ref(self.cx.storage.alloc(|| {
                RefTy::new(
                    data,
                    to_api_lifetime(self.cx, rust_lt),
                    to_api_mutability(mut_ty.mutbl),
                    self.conv_syn_ty(mut_ty.ty),
                )
            })),
            hir::TyKind::BareFn(rust_fn) => self.conv_syn_bare_fn(data, rust_fn),
            hir::TyKind::Never => TyKind::Never(self.cx.storage.alloc(|| NeverTy::new(data))),
            hir::TyKind::Tup(rustc_tys) => {
                let api_tys = self
                    .cx
                    .storage
                    .alloc_slice_iter(rustc_tys.iter().map(|rustc_ty| self.conv_syn_ty(rustc_ty)));
                TyKind::Tuple(self.cx.storage.alloc(|| TupleTy::new(data, api_tys)))
            },
            hir::TyKind::Path(qpath) => self.conv_syn_qpath(data, qpath),
            hir::TyKind::OpaqueDef(_, _, _) => {
                // This requires function items to be implemented. Therefore we'll leave this as an open TODO for
                // now
                todo!("{:#?}", rustc_ty)
            },
            hir::TyKind::TraitObject(rust_bounds, rust_lt, _syntax) => TyKind::TraitObj(
                self.cx
                    .storage
                    .alloc(|| TraitObjTy::new(data, to_api_trait_bounds_from_hir(self.cx, rust_bounds, rust_lt))),
            ),
            hir::TyKind::Infer => TyKind::Inferred(self.cx.storage.alloc(|| InferredTy::new(data))),
            hir::TyKind::Err => unreachable!("would have triggered a rustc error"),
            hir::TyKind::Typeof(_) => unreachable!("docs state: 'Unused for now.'"),
        }
    }

    fn conv_syn_qpath(&self, data: CommonTyData<'ast>, qpath: &hir::QPath<'tcx>) -> TyKind<'ast> {
        // FIXME: These `todo!()`s are currently not testable with the limited items
        // that are in the API. Therefore we'll leave them for another day.
        match qpath {
            hir::QPath::Resolved(None, path) => match path.res {
                hir::def::Res::Def(
                    hir::def::DefKind::Enum | hir::def::DefKind::Struct | hir::def::DefKind::Union,
                    id,
                ) => self.conv_syn_ty_adt_id(data, id, path.segments.last().and_then(|s| s.args)),
                hir::def::Res::Def(
                    hir::def::DefKind::LifetimeParam | hir::def::DefKind::TyParam | hir::def::DefKind::ConstParam,
                    id,
                ) => TyKind::Generic(self.cx.storage.alloc(|| GenericTy::new(data, to_generic_id(id)))),
                hir::def::Res::Def(hir::def::DefKind::TyAlias, id) => {
                    TyKind::Alias(self.cx.storage.alloc(|| AliasTy::new(data, to_item_id(id))))
                },
                hir::def::Res::PrimTy(prim_ty) => self.conv_syn_prim_ty(data, prim_ty),
                hir::def::Res::SelfTyParam { trait_: def_id, .. }
                | hir::def::Res::SelfTyAlias { alias_to: def_id, .. } => {
                    TyKind::SelfTy(self.cx.storage.alloc(|| SelfTy::new(data, to_item_id(def_id))))
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
                self.cx
                    .storage
                    .alloc(|| RelativeTy::new(data, self.conv_syn_ty(ty), to_symbol_id(segment.ident.name))),
            ),
            hir::QPath::LangItem(_, _, _) => todo!(),
        }
    }

    fn conv_syn_ty_adt_id(
        &self,
        data: CommonTyData<'ast>,
        adt_id: hir::def_id::DefId,
        rustc_args: Option<&'tcx hir::GenericArgs<'tcx>>,
    ) -> TyKind<'ast> {
        let adt = self.cx.rustc_cx.adt_def(adt_id);
        // You were working on ADT parsing to then do all types in Option<...>. Next you need to
        // fill in the variants and generic args. The second one will be more work but also will be more
        // important
        let def_id = to_ty_def_id(adt_id);
        let generic_args = to_api_generic_args(self.cx, rustc_args);
        match adt.adt_kind() {
            rustc_middle::ty::AdtKind::Struct => TyKind::Struct(
                self.cx
                    .storage
                    .alloc(|| StructTy::new(data, def_id, generic_args, adt.is_variant_list_non_exhaustive())),
            ),
            rustc_middle::ty::AdtKind::Enum => TyKind::Enum(
                self.cx
                    .storage
                    .alloc(|| EnumTy::new(data, def_id, generic_args, adt.is_variant_list_non_exhaustive())),
            ),
            rustc_middle::ty::AdtKind::Union => {
                TyKind::Union(self.cx.storage.alloc(|| UnionTy::new(data, def_id, generic_args)))
            },
        }
    }

    fn conv_syn_bare_fn(&self, data: CommonTyData<'ast>, rust_fn: &hir::BareFnTy<'tcx>) -> TyKind<'ast> {
        assert_eq!(rust_fn.param_names.len(), rust_fn.decl.inputs.len());
        let params = rust_fn
            .decl
            .inputs
            .iter()
            .zip(rust_fn.param_names.iter())
            .map(|(rustc_ty, name)| {
                Parameter::new(
                    Some(to_symbol_id(name.name)),
                    Some(self.conv_syn_ty(rustc_ty)),
                    Some(to_span_id(name.span)),
                )
            });
        let params = self.cx.storage.alloc_slice_iter(params);
        let return_ty = if let hir::FnRetTy::Return(rust_ty) = rust_fn.decl.output {
            Some(self.conv_syn_ty(rust_ty))
        } else {
            None
        };
        TyKind::Fn(self.cx.storage.alloc(|| {
            FnTy::new(
                data,
                CommonCallableData::new(
                    false,
                    false,
                    matches!(rust_fn.unsafety, hir::Unsafety::Unsafe),
                    false,
                    to_api_abi(rust_fn.abi),
                    false,
                    params,
                    return_ty,
                ),
            )
        }))
    }

    fn conv_syn_prim_ty(&self, data: CommonTyData<'ast>, prim_ty: hir::PrimTy) -> TyKind<'ast> {
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
            hir::PrimTy::Str => return TyKind::Text(self.cx.storage.alloc(|| TextTy::new(data, TextKind::Str))),
            hir::PrimTy::Bool => return TyKind::Bool(self.cx.storage.alloc(|| BoolTy::new(data))),
            hir::PrimTy::Char => {
                return TyKind::Text(self.cx.storage.alloc(|| TextTy::new(data, TextKind::Char)));
            },
        };
        TyKind::Num(self.cx.storage.alloc(|| NumTy::new(data, num_kind)))
    }
}
