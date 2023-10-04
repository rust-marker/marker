use crate::{
    ast::expr::ConstExpr,
    ffi::{FfiOption, FfiSlice},
};

use super::{CommonSynTyData, TyKind};

/// The syntactic representation of a tuple type like [`()`](prim@tuple) or [`(T, U)`](prim@tuple)
#[repr(C)]
#[derive(Debug)]
pub struct TupleTy<'ast> {
    data: CommonSynTyData<'ast>,
    types: FfiSlice<'ast, TyKind<'ast>>,
}

#[cfg(feature = "driver-api")]
impl<'ast> TupleTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, types: &'ast [TyKind<'ast>]) -> Self {
        Self {
            data,
            types: types.into(),
        }
    }
}

super::impl_ty_data!(TupleTy<'ast>, Tuple);

impl<'ast> TupleTy<'ast> {
    pub fn types(&self) -> &[TyKind<'ast>] {
        self.types.as_slice()
    }
}

impl<'ast> std::fmt::Display for TupleTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_tuple("");

        for entry in self.types.as_slice() {
            f.field(entry);
        }

        f.finish()
    }
}

/// The syntactic representation of a variable length slice like [`[T]`](prim@slice)
#[repr(C)]
#[derive(Debug)]
pub struct SliceTy<'ast> {
    data: CommonSynTyData<'ast>,
    inner_ty: TyKind<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> SliceTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, inner_ty: TyKind<'ast>) -> Self {
        Self { data, inner_ty }
    }
}

super::impl_ty_data!(SliceTy<'ast>, Slice);

impl<'ast> SliceTy<'ast> {
    pub fn inner_ty(&self) -> TyKind<'ast> {
        self.inner_ty
    }
}

impl<'ast> std::fmt::Display for SliceTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(std::iter::once(self.inner_ty())).finish()
    }
}

/// The syntactic representation of an array with a known size like: [`[T; N]`](prim@array)
#[repr(C)]
#[derive(Debug)]
pub struct ArrayTy<'ast> {
    data: CommonSynTyData<'ast>,
    inner_ty: TyKind<'ast>,
    // FIXME(xFrednet): This might need to change, if a syntax like `[1; _]` is
    // ever supported, as proposed in https://github.com/rust-lang/rust/issues/85077
    len: FfiOption<ConstExpr<'ast>>,
}

super::impl_ty_data!(ArrayTy<'ast>, Array);

impl<'ast> ArrayTy<'ast> {
    pub fn inner_ty(&self) -> TyKind<'ast> {
        self.inner_ty
    }

    pub fn len(&self) -> Option<&ConstExpr<'ast>> {
        self.len.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ArrayTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, inner_ty: TyKind<'ast>, len: Option<ConstExpr<'ast>>) -> Self {
        Self {
            data,
            inner_ty,
            len: len.into(),
        }
    }
}

impl<'ast> std::fmt::Display for ArrayTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // FIXME: Add length expression
        f.debug_list().entries(std::iter::once(self.inner_ty())).finish()
    }
}
