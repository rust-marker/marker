mod fn_ty;
mod other_ty;
mod prim_ty;
mod ptr_ty;
mod sequence_ty;
mod trait_ty;
mod user_ty;

pub use fn_ty::*;
pub use other_ty::*;
pub use prim_ty::*;
pub use ptr_ty::*;
pub use sequence_ty::*;
pub use trait_ty::*;
pub use user_ty::*;

use crate::common::DriverTyId;
use std::{fmt::Debug, marker::PhantomData};

/// The semantic representation of a type.
#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum TyKind<'ast> {
    // ================================
    // Primitive types
    // ================================
    /// The `bool` type
    Bool(&'ast BoolTy<'ast>),
    /// A numeric type like [`u32`], [`i32`], [`f64`]
    Num(&'ast NumTy<'ast>),
    /// A textual type like [`char`] or [`str`]
    Text(&'ast TextTy<'ast>),
    /// The never type [`!`](prim@never)
    Never(&'ast NeverTy<'ast>),
    // ================================
    // Sequence types
    // ================================
    /// A tuple type like [`()`](prim@tuple), [`(T, U)`](prim@tuple)
    Tuple(&'ast TupleTy<'ast>),
    /// An array with a known size like: [`[T; N]`](prim@array)
    Array(&'ast ArrayTy<'ast>),
    /// A variable length slice like [`[T]`](prim@slice)
    Slice(&'ast SliceTy<'ast>),
    // ================================
    // Function types
    // ================================
    /// A [function item type](https://doc.rust-lang.org/reference/types/function-item.html)
    /// identifying a specific function and potentualy additional generics.
    Fn(&'ast FnTy<'ast>),
    /// The semantic representation of a
    /// [closure type](https://doc.rust-lang.org/reference/types/closure.html).
    Closure(&'ast ClosureTy<'ast>),
    // ================================
    // Pointer types
    // ================================
    /// A reference like [`&T`](prim@reference) or [`&mut T`](prim@reference)
    Ref(&'ast RefTy<'ast>),
    /// A raw pointer like [`*const T`](prim@pointer) or [`*mut T`](prim@pointer)
    RawPtr(&'ast RawPtrTy<'ast>),
    /// The semantic representation of a function pointer, like [`fn (T) -> U`](prim@fn)
    FnPtr(&'ast FnPtrTy<'ast>),
    // ================================
    // Trait types
    // ================================
    /// A trait object like [`dyn Trait`](https://doc.rust-lang.org/stable/std/keyword.dyn.html)
    TraitObj(&'ast TraitObjTy<'ast>),
    // ================================
    // User defined types
    // ================================
    /// A user defined data type, identified by an [`TyDefId`](crate::common::TyDefId)
    Adt(&'ast AdtTy<'ast>),
    /// A generic type defined by a generic parameter
    Generic(&'ast GenericTy<'ast>),
    /// A type alias. Note that simple type aliases will already be replaced in
    /// semantic types. This kind is mainly used for type aliases, where the concrete
    /// type is not yet known, for example in traits.
    Alias(&'ast AliasTy<'ast>),
    // ================================
    // Other types
    // ================================
    /// The placeholder type, signalling that the semantic type is still unstable
    /// and therefor not represented as part of the API.
    Unstable(&'ast UnstableTy<'ast>),
}

impl<'ast> TyKind<'ast> {
    /// Peel off all reference types in this type until there are none left.
    ///
    /// This method is idempotent, i.e. `ty.peel_refs().peel_refs() == ty.peel_refs()`.
    ///
    /// # Examples
    ///
    /// - `u8` -> `u8`
    /// - `&'a mut u8` -> `u8`
    /// - `&'a &'b u8` -> `u8`
    /// - `&'a *const &'b u8 -> *const &'b u8`
    ///
    /// # Acknowledgements
    ///
    /// This method was based on rustc's internal [`peel_refs`] method.
    ///
    /// [`peel_refs`]: https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.Ty.html#method.peel_refs
    #[must_use]
    pub fn peel_refs(self) -> Self {
        // XXX: exactly the same `peel_refs` method exists on `ast::TyKind`.
        // If you modify this method here, please check if the modifications
        // should also apply to `ast::TyKind` as well.

        let mut ty = self;
        while let Self::Ref(ref_ty) = ty {
            ty = ref_ty.inner_ty();
        }
        ty
    }
}

#[repr(C)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub(crate) struct CommonTyData<'ast> {
    #[cfg_attr(feature = "driver-api", builder(default))]
    _lifetime: PhantomData<&'ast ()>,
    driver_id: DriverTyId,
}

impl<'ast> Debug for CommonTyData<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommonTyData {...}").finish()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> CommonTyData<'ast> {
    pub fn driver_id(&self) -> DriverTyId {
        self.driver_id
    }
}

macro_rules! impl_ty_data {
    ($self_ty:ty, $enum_name:ident) => {
        #[cfg(feature = "driver_api")]
        impl<'ast> $self_ty {
            pub fn data(&self) -> &$crate::sem::ty::CommonTyData<'ast> {
                &self.data
            }
        }

        impl<'ast> From<&'ast $self_ty> for $crate::sem::ty::TyKind<'ast> {
            fn from(from: &'ast $self_ty) -> Self {
                $crate::sem::ty::TyKind::$enum_name(from)
            }
        }
    };
}
use impl_ty_data;
