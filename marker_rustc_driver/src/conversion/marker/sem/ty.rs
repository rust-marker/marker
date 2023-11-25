use marker_api::{
    common::{NumKind, TextKind},
    sem::{
        self, AdtTy, AliasTy, ArrayTy, BoolTy, ClosureTy, ConstValue, FnPtrTy, FnTy, GenericTy, NeverTy, NumTy,
        RawPtrTy, RefTy, SliceTy, TextTy, TraitObjTy, TupleTy, TyKind, UnstableTy,
    },
};
use rustc_middle as mid;

use crate::conversion::marker::MarkerConverterInner;

impl<'ast, 'tcx> MarkerConverterInner<'ast, 'tcx> {
    #[must_use]
    pub fn to_sem_ty(&self, rustc_ty: mid::ty::Ty<'tcx>) -> TyKind<'ast> {
        let data = sem::CommonTyData::builder()
            .driver_id(self.to_driver_ty_id(rustc_ty))
            .build();

        // Semantic types could be cached, the question is if they should and at
        // which level.
        match &rustc_ty.kind() {
            mid::ty::TyKind::Bool => TyKind::Bool(self.alloc(BoolTy::builder().data(data).build())),
            mid::ty::TyKind::Int(int_ty) => {
                let num_ty = match int_ty {
                    mid::ty::IntTy::Isize => NumKind::Isize,
                    mid::ty::IntTy::I8 => NumKind::I8,
                    mid::ty::IntTy::I16 => NumKind::I16,
                    mid::ty::IntTy::I32 => NumKind::I32,
                    mid::ty::IntTy::I64 => NumKind::I64,
                    mid::ty::IntTy::I128 => NumKind::I128,
                };
                TyKind::Num(self.alloc(NumTy::builder().data(data).numeric_kind(num_ty).build()))
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
                TyKind::Num(self.alloc(NumTy::builder().data(data).numeric_kind(num_ty).build()))
            },
            mid::ty::TyKind::Float(float_ty) => {
                let num_ty = match float_ty {
                    mid::ty::FloatTy::F32 => NumKind::F32,
                    mid::ty::FloatTy::F64 => NumKind::F64,
                };
                TyKind::Num(self.alloc(NumTy::builder().data(data).numeric_kind(num_ty).build()))
            },
            mid::ty::TyKind::Char => {
                TyKind::Text(self.alloc(TextTy::builder().data(data).textual_kind(TextKind::Char).build()))
            },
            mid::ty::TyKind::Str => {
                TyKind::Text(self.alloc(TextTy::builder().data(data).textual_kind(TextKind::Str).build()))
            },
            mid::ty::TyKind::Adt(def, generics) => TyKind::Adt(
                self.alloc(
                    AdtTy::builder()
                        .data(data)
                        .def_id(self.to_ty_def_id(def.did()))
                        .generics(self.to_sem_generic_args(generics))
                        .build(),
                ),
            ),
            mid::ty::TyKind::Foreign(_) => {
                todo!("foreign type are currently sadly not supported. See rust-marker/marker#182")
            },
            mid::ty::TyKind::Array(inner, _len) => TyKind::Array(
                self.alloc(
                    ArrayTy::builder()
                        .data(data)
                        .inner_ty(self.to_sem_ty(*inner))
                        .len(ConstValue::new())
                        .build(),
                ),
            ),
            mid::ty::TyKind::Slice(inner) => {
                TyKind::Slice(self.alloc(SliceTy::builder().data(data).inner_ty(self.to_sem_ty(*inner)).build()))
            },
            mid::ty::TyKind::Tuple(ty_lst) => TyKind::Tuple(
                self.alloc(
                    TupleTy::builder()
                        .data(data)
                        .types(self.alloc_slice(ty_lst.iter().map(|ty| self.to_sem_ty(ty))))
                        .build(),
                ),
            ),
            mid::ty::TyKind::RawPtr(ty_and_mut) => TyKind::RawPtr(
                self.alloc(
                    RawPtrTy::builder()
                        .data(data)
                        .mutability(self.to_mutability(ty_and_mut.mutbl))
                        .inner_ty(self.to_sem_ty(ty_and_mut.ty))
                        .build(),
                ),
            ),
            mid::ty::TyKind::Ref(_lifetime, inner, muta) => TyKind::Ref(
                self.alloc(
                    RefTy::builder()
                        .data(data)
                        .mutability(self.to_mutability(*muta))
                        .inner_ty(self.to_sem_ty(*inner))
                        .build(),
                ),
            ),
            mid::ty::TyKind::FnDef(fn_id, generic_args) => TyKind::Fn(
                self.alloc(
                    FnTy::builder()
                        .data(data)
                        .fn_id(self.to_item_id(*fn_id))
                        .generics(self.to_sem_generic_args(generic_args))
                        .build(),
                ),
            ),
            mid::ty::TyKind::FnPtr(fn_info) => TyKind::FnPtr(
                self.alloc(
                    FnPtrTy::builder()
                        .data(data)
                        .safety(self.to_safety(fn_info.unsafety()))
                        .abi(self.to_abi(fn_info.abi()))
                        .params(
                            self.alloc_slice(
                                fn_info
                                    .inputs()
                                    .skip_binder()
                                    .iter()
                                    .map(|input| self.to_sem_ty(*input)),
                            ),
                        )
                        .return_ty(self.to_sem_ty(fn_info.output().skip_binder()))
                        .build(),
                ),
            ),
            mid::ty::TyKind::Dynamic(binders, _region, kind) => {
                if !matches!(kind, mid::ty::DynKind::Dyn) {
                    unimplemented!("the docs are not totally clear, when `DynStar` is used, her it is: {rustc_ty:#?}")
                }
                TyKind::TraitObj(
                    self.alloc(
                        TraitObjTy::builder()
                            .data(data)
                            .bounds(self.to_sem_trait_bounds(binders))
                            .build(),
                    ),
                )
            },
            mid::ty::TyKind::Closure(id, generics) => TyKind::Closure(
                self.alloc(
                    ClosureTy::builder()
                        .data(data)
                        .def_id(self.to_ty_def_id(*id))
                        .generics(self.to_sem_generic_args(generics))
                        .build(),
                ),
            ),
            mid::ty::TyKind::Coroutine(_, _, _) | mid::ty::TyKind::CoroutineWitness(_, _) => {
                TyKind::Unstable(self.alloc(UnstableTy::builder().data(data).build()))
            },
            mid::ty::TyKind::Never => TyKind::Never(self.alloc(NeverTy::builder().data(data).build())),
            mid::ty::TyKind::Alias(_, info) => TyKind::Alias(
                self.alloc(
                    AliasTy::builder()
                        .data(data)
                        .alias_item(self.to_item_id(info.def_id))
                        .build(),
                ),
            ),
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
                TyKind::Generic(
                    self.alloc(
                        GenericTy::builder()
                            .data(data)
                            .generic_id(self.to_generic_id(generic_info.def_id))
                            .build(),
                    ),
                )
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
