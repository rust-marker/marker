use crate::{
    ast::{generic::Lifetime, impl_callable_data_trait, Abi, CommonCallableData, Mutability, Safety},
    ffi::{FfiOption, FfiSlice},
};

use super::{CommonSynTyData, SemTyKind, SynTyKind};

/// The syntactic representation of a reference like [`&T`](prim@reference)
/// or [`&mut T`](prim@reference)

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SynRefTy<'ast> {
    data: CommonSynTyData<'ast>,
    lifetime: FfiOption<Lifetime<'ast>>,
    is_mut: bool,
    inner_ty: SynTyKind<'ast>,
}

impl<'ast> SynRefTy<'ast> {
    pub fn has_lifetime(&self) -> bool {
        self.lifetime.get().is_some()
    }

    pub fn is_mut(&self) -> bool {
        self.is_mut
    }

    pub fn inner_ty(&self) -> SynTyKind<'ast> {
        self.inner_ty
    }
}

super::impl_ty_data!(SynRefTy<'ast>, Ref);

#[cfg(feature = "driver-api")]
impl<'ast> SynRefTy<'ast> {
    pub fn new(
        data: CommonSynTyData<'ast>,
        lifetime: Option<Lifetime<'ast>>,
        is_mut: bool,
        inner_ty: SynTyKind<'ast>,
    ) -> Self {
        Self {
            data,
            lifetime: lifetime.into(),
            is_mut,
            inner_ty,
        }
    }
}

/// The semantic representation of a reference like [`&T`](prim@reference)
/// or [`&mut T`](prim@reference)
///
/// Note that the semantic representation doesn't contain lifetime information.
/// Marker currently doesn't support the analysis of lifetimes. Removing them
/// from the type also simplifies type comparisons.
#[repr(C)]
#[derive(Debug)]
pub struct SemRefTy<'ast> {
    mutability: Mutability,
    inner_ty: SemTyKind<'ast>,
}

impl<'ast> SemRefTy<'ast> {
    /// This returns the [`Mutability`] of the referenced type.
    pub fn mutability(&self) -> Mutability {
        self.mutability
    }

    /// This returns the inner [`SemTyKind`]
    pub fn inner_ty(&self) -> SemTyKind<'ast> {
        self.inner_ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemRefTy<'ast> {
    pub fn new(mutability: Mutability, inner_ty: SemTyKind<'ast>) -> Self {
        Self { mutability, inner_ty }
    }
}

/// The syntactic representation of a raw pointer like [`*const T`](prim@pointer)
/// or [`*mut T`](prim@pointer)
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SynRawPtrTy<'ast> {
    data: CommonSynTyData<'ast>,
    is_mut: bool,
    inner_ty: SynTyKind<'ast>,
}

impl<'ast> SynRawPtrTy<'ast> {
    pub fn is_mut(&self) -> bool {
        self.is_mut
    }

    pub fn inner_ty(&self) -> SynTyKind<'ast> {
        self.inner_ty
    }
}

super::impl_ty_data!(SynRawPtrTy<'ast>, RawPtr);

#[cfg(feature = "driver-api")]
impl<'ast> SynRawPtrTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, is_mut: bool, inner_ty: SynTyKind<'ast>) -> Self {
        Self { data, is_mut, inner_ty }
    }
}

/// The semantic representation of a raw pointer like [`*const T`](prim@pointer)
/// or [`*mut T`](prim@pointer)
#[repr(C)]
#[derive(Debug)]
pub struct SemRawPtrTy<'ast> {
    mutability: Mutability,
    inner_ty: SemTyKind<'ast>,
}

impl<'ast> SemRawPtrTy<'ast> {
    pub fn mutability(&self) -> Mutability {
        self.mutability
    }

    pub fn inner_ty(&self) -> SemTyKind<'ast> {
        self.inner_ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemRawPtrTy<'ast> {
    pub fn new(mutability: Mutability, inner_ty: SemTyKind<'ast>) -> Self {
        Self { mutability, inner_ty }
    }
}

/// The semantic representation of a function pointer, like [`fn (T) -> U`](prim@fn)
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SynFnPtrTy<'ast> {
    data: CommonSynTyData<'ast>,
    callable_data: CommonCallableData<'ast>,
    // FIXME: Add `for<'a>` bound
}

#[cfg(feature = "driver-api")]
impl<'ast> SynFnPtrTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, callable_data: CommonCallableData<'ast>) -> Self {
        Self { data, callable_data }
    }
}

super::impl_ty_data!(SynFnPtrTy<'ast>, FnPtr);
impl_callable_data_trait!(SynFnPtrTy<'ast>);

/// The semantic representation of a function pointer, like [`fn (T) -> U`](prim@fn)
#[repr(C)]
#[derive(Debug)]
pub struct SemFnPtrTy<'ast> {
    safety: Safety,
    abi: Abi,
    params: FfiSlice<'ast, SemTyKind<'ast>>,
    return_ty: SemTyKind<'ast>,
}

impl<'ast> SemFnPtrTy<'ast> {
    pub fn safety(&self) -> Safety {
        self.safety
    }

    pub fn abi(&self) -> Abi {
        self.abi
    }

    pub fn params(&self) -> &[SemTyKind<'ast>] {
        self.params.get()
    }

    pub fn return_ty(&self) -> SemTyKind<'ast> {
        self.return_ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemFnPtrTy<'ast> {
    pub fn new(safety: Safety, abi: Abi, params: &'ast [SemTyKind<'ast>], return_ty: SemTyKind<'ast>) -> Self {
        Self {
            safety,
            abi,
            params: params.into(),
            return_ty,
        }
    }
}
