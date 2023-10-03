use marker_api::{
    common::{NumKind, TextKind},
    sem::{
        AdtTy, AliasTy, ArrayTy, BoolTy, ClosureTy, ConstValue, FnPtrTy, FnTy, GenericTy, NeverTy, NumTy, RawPtrTy,
        RefTy, SliceTy, TextTy, TraitObjTy, TupleTy, TyKind, UnstableTy,
    },
};
use rustc_middle as mid;

use crate::conversion::marker::MarkerConverterInner;

impl<'ast, 'tcx> MarkerConverterInner<'ast, 'tcx> {
    #[must_use]
    pub fn to_sem_ty(&self, rustc_ty: mid::ty::Ty<'tcx>) -> TyKind<'ast> {
        // Semantic types could be cached, the question is if they should and at
        // which level.
        match &rustc_ty.kind() {
            mid::ty::TyKind::Bool => TyKind::Bool(self.alloc(BoolTy::new())),
            mid::ty::TyKind::Char => TyKind::Text(self.alloc(TextTy::new(TextKind::Char))),
            mid::ty::TyKind::Int(int_ty) => {
                let num_ty = match int_ty {
                    mid::ty::IntTy::Isize => NumKind::Isize,
                    mid::ty::IntTy::I8 => NumKind::I8,
                    mid::ty::IntTy::I16 => NumKind::I16,
                    mid::ty::IntTy::I32 => NumKind::I32,
                    mid::ty::IntTy::I64 => NumKind::I64,
                    mid::ty::IntTy::I128 => NumKind::I128,
                };
                TyKind::Num(self.alloc(NumTy::new(num_ty)))
            },
            mid::ty::TyKind::Uint(uint_ty) => {
                let num_ty = match uint_ty {
                    mid::ty::UintTy::Usize => NumKind::Usize,
                    mid::ty::UintTy::U8 => NumKind::U8,
                    mid::ty::UintTy::U16 => NumKind::U16,
                    mid::ty::UintTy::U32 => NumKind::U32,
                    mid::ty::UintTy::U64 => NumKind::U64,
                    mid::ty::UintTy::U128 => NumKind::U128,
                };
                TyKind::Num(self.alloc(NumTy::new(num_ty)))
            },
            mid::ty::TyKind::Float(float_ty) => {
                let num_ty = match float_ty {
                    mid::ty::FloatTy::F32 => NumKind::F32,
                    mid::ty::FloatTy::F64 => NumKind::F64,
                };
                TyKind::Num(self.alloc(NumTy::new(num_ty)))
            },
            mid::ty::TyKind::Str => TyKind::Text(self.alloc(TextTy::new(TextKind::Str))),
            mid::ty::TyKind::Adt(def, generics) => TyKind::Adt(self.alloc(AdtTy::new(
                self.to_ty_def_id(def.did()),
                self.to_sem_generic_args(generics),
            ))),
            mid::ty::TyKind::Foreign(_) => {
                todo!("foreign type are currently sadly not supported. See rust-marker/marker#182")
            },
            mid::ty::TyKind::Array(inner, _len) => {
                TyKind::Array(self.alloc(ArrayTy::new(self.to_sem_ty(*inner), ConstValue::new())))
            },
            mid::ty::TyKind::Slice(inner) => TyKind::Slice(self.alloc(SliceTy::new(self.to_sem_ty(*inner)))),
            mid::ty::TyKind::Tuple(ty_lst) => TyKind::Tuple(self.alloc(TupleTy::new(
                self.alloc_slice(ty_lst.iter().map(|ty| self.to_sem_ty(ty))),
            ))),
            mid::ty::TyKind::RawPtr(ty_and_mut) => TyKind::RawPtr(self.alloc(RawPtrTy::new(
                self.to_mutability(ty_and_mut.mutbl),
                self.to_sem_ty(ty_and_mut.ty),
            ))),
            mid::ty::TyKind::Ref(_lifetime, inner, muta) => {
                TyKind::Ref(self.alloc(RefTy::new(self.to_mutability(*muta), self.to_sem_ty(*inner))))
            },
            mid::ty::TyKind::FnDef(fn_id, generic_args) => TyKind::FnTy(self.alloc(FnTy::new(
                self.to_item_id(*fn_id),
                self.to_sem_generic_args(generic_args),
            ))),
            mid::ty::TyKind::FnPtr(fn_info) => TyKind::FnPtr(
                self.alloc(FnPtrTy::new(
                    self.to_safety(fn_info.unsafety()),
                    self.to_abi(fn_info.abi()),
                    self.alloc_slice(
                        fn_info
                            .inputs()
                            .skip_binder()
                            .iter()
                            .map(|input| self.to_sem_ty(*input)),
                    ),
                    self.to_sem_ty(fn_info.output().skip_binder()),
                )),
            ),
            mid::ty::TyKind::Dynamic(binders, _region, kind) => {
                if !matches!(kind, mid::ty::DynKind::Dyn) {
                    unimplemented!("the docs are not totally clear, when `DynStar` is used, her it is: {rustc_ty:#?}")
                }
                TyKind::TraitObj(self.alloc(TraitObjTy::new(self.to_sem_trait_bounds(binders))))
            },
            mid::ty::TyKind::Closure(id, generics) => TyKind::ClosureTy(self.alloc(ClosureTy::new(
                self.to_ty_def_id(*id),
                self.to_sem_generic_args(generics),
            ))),
            mid::ty::TyKind::Generator(_, _, _)
            | mid::ty::TyKind::GeneratorWitness(_)
            | mid::ty::TyKind::GeneratorWitnessMIR(_, _) => TyKind::Unstable(self.alloc(UnstableTy::new())),
            mid::ty::TyKind::Never => TyKind::Never(self.alloc(NeverTy::new())),
            mid::ty::TyKind::Alias(_, info) => TyKind::Alias(self.alloc(AliasTy::new(self.to_item_id(info.def_id)))),
            mid::ty::TyKind::Param(param) => {
                let body_id = self
                    .rustc_body
                    .borrow()
                    .expect("semantic `TyKind::Param` is only valid inside bodies");
                // This is a local id, this makes sense, since rustc only accesses
                // expressions and therefore semantic types of the current crate.
                // This should be fine...
                let owner = self.rustc_cx.hir().body_owner_def_id(body_id);
                let generic_info = self
                    .rustc_cx
                    .generics_of(owner.to_def_id())
                    .type_param(param, self.rustc_cx);
                TyKind::Generic(self.alloc(GenericTy::new(self.to_generic_id(generic_info.def_id))))
            },
            mid::ty::TyKind::Bound(_, _) => {
                unreachable!("used by rustc for higher ranked types, which are not represented in marker")
            },
            mid::ty::TyKind::Placeholder(_) | mid::ty::TyKind::Infer(_) => {
                unreachable!("used by rustc during typechecking, should not exist afterwards")
            },
            mid::ty::TyKind::Error(_) => unreachable!("would have triggered a rustc error"),
        }
    }
}
