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

use crate::{
    common::{DriverTyId, TyDefId},
    context::with_cx,
    ffi::FfiSlice,
};
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

    /// This function tests if a given type implements a trait, specified by
    /// the given [`TestTraitRef`]. A [`TraitTestMode`] can be specified as
    /// part of the trait reference.
    #[must_use]
    pub fn implements_trait(self, trait_ref: &TestTraitRef) -> bool {
        let check_with_ids = |ids: &[TyDefId]| {
            let ffi = FfiTestTraitRef {
                trait_ids: ids.into(),
                mode: trait_ref.mode,
            };
            with_cx(&self, |cx| cx.ty_implements_trait(self, &ffi))
        };

        match &trait_ref.trait_ref {
            TyDefIdSource::Path(path) => check_with_ids(with_cx(&self, |cx| cx.resolve_ty_ids(path))),
            TyDefIdSource::Id(id) => check_with_ids(&[*id]),
        }
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> TyKind<'ast> {
    impl_ty_kind_fn!(data() -> &CommonTyData<'ast>);
}

macro_rules! impl_ty_kind_fn {
    ($method:ident () -> $return_ty:ty) => {
        impl_ty_kind_fn!($method() -> $return_ty,
            Bool, Num, Text, Never,
            Tuple, Array, Slice,
            Fn, Closure,
            Ref, RawPtr, FnPtr,
            TraitObj, Adt, Generic, Alias,
            Unstable
        );
    };
    ($method:ident () -> $return_ty:ty $(, $item:ident)+) => {
        pub fn $method(&self) -> $return_ty {
            match self {
                $(TyKind::$item(data) => data.$method(),)*
            }
        }
    };
}
use impl_ty_kind_fn;

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
        #[cfg(feature = "driver-api")]
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

/// This struct describes a trait and its generic arguments. It is used to check
/// if a semantic type implements a trait.
///
/// See [`TyKind::implements_trait`].
#[derive(Debug)]
pub struct TestTraitRef {
    trait_ref: TyDefIdSource,
    // TODO generics
    mode: TraitTestMode,
}

impl TestTraitRef {
    pub fn builder() -> TestTraitRefBuilder {
        TestTraitRefBuilder::new()
    }
}

/// A builder used to construct a [`TestTraitRef`] instance.
#[derive(Debug)]
pub struct TestTraitRefBuilder {
    trait_ref: Option<TyDefIdSource>,
    mode: TraitTestMode,
}

impl TestTraitRefBuilder {
    fn new() -> Self {
        Self {
            trait_ref: None,
            mode: TraitTestMode::default(),
        }
    }

    pub fn trait_from_path(&mut self, path: impl Into<String>) -> &mut Self {
        self.trait_ref = Some(TyDefIdSource::Path(path.into()));
        self
    }

    pub fn trait_from_id(&mut self, id: impl Into<TyDefId>) -> &mut Self {
        self.trait_ref = Some(TyDefIdSource::Id(id.into()));
        self
    }

    pub fn mode(&mut self, mode: TraitTestMode) -> &mut Self {
        self.mode = mode;
        self
    }

    pub fn build(&mut self) -> TestTraitRef {
        TestTraitRef {
            trait_ref: self.trait_ref.take().expect("the trait was never set"),
            mode: self.mode,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
pub(crate) struct FfiTestTraitRef<'a> {
    trait_ids: FfiSlice<'a, TyDefId>,
    // TODO generics
    mode: TraitTestMode,
}

#[cfg(feature = "driver-api")]
impl<'a> FfiTestTraitRef<'a> {
    pub fn trait_ids(&self) -> &'a [TyDefId] {
        self.trait_ids.get()
    }

    pub fn mode(&self) -> TraitTestMode {
        self.mode
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub enum TyDefIdSource {
    Id(TyDefId),
    // TODO: Handle the path properly, since `Str` is not ABI safe
    Path(String),
}

impl From<TyDefId> for TyDefIdSource {
    fn from(value: TyDefId) -> Self {
        TyDefIdSource::Id(value)
    }
}

impl From<String> for TyDefIdSource {
    fn from(value: String) -> Self {
        TyDefIdSource::Path(value)
    }
}

/// This enum defines how strict the [`TyKind::implements_trait`] test should be.
/// The difference is probably best illustrated by examples. For that, let's
/// take the following traits:
///
/// ```
/// // Definitions
/// trait SimpleTrait { /* ... */ }
/// trait GenericTrait<T> { /* ... */ }
/// trait TwoGenericTrait<T, U=u8> { /* ... */ }
/// ```
///
/// Now we have a `SimpleType` which implements a few traits:
///
/// ```
/// # trait SimpleTrait { /* ... */ }
/// # trait GenericTrait<T> { /* ... */ }
/// # trait TwoGenericTrait<T, U=u8> { /* ... */ }
/// struct SimpleType;
///
/// impl SimpleTrait for SimpleType {}
/// impl GenericTrait<i16> for SimpleType {}
/// // The second generic argument, defaults to `u8`.
/// impl TwoGenericTrait<u16> for SimpleType {}
/// ```
///
/// | Semantic type and checked trait                           | Lossy | Strict |
/// | --------------------------------------------------------- | ----- | ------ |
/// | `SimpleType` implements `SimpleTrait`                     | ✅     | ✅      |
/// | `SimpleType` implements `GenericTrait`                    | ✅     | ❌      |
/// | `SimpleType` implements `GenericTrait<!>`                 | ❌     | ❌      |
/// | `SimpleType` implements `TwoGenericTrait<u16>`            | ✅     | ❌      |
/// | `SimpleType` implements `TwoGenericTrait<u16, u8>`        | ✅     | ✅      |
/// | `SimpleType` implements `TwoGenericTrait<u16, !>`         | ❌     | ❌      |
///
/// We can also check traits for types with generic parameter:
///
/// ```
/// # trait SimpleTrait { /* ... */ }
/// # trait GenericTrait<T> { /* ... */ }
/// # trait TwoGenericTrait<T, U=u8> { /* ... */ }
/// struct GenericType<T>(T);
///
/// impl<T> SimpleTrait for GenericType<T> {}
/// impl GenericTrait<u32> for GenericType<u32> {}
/// impl TwoGenericTrait<i32, i32> for GenericType<u32> {}
/// ```
/// | Semantic type and checked trait                           | Lossy | Strict |
/// | --------------------------------------------------------- | ----- | ------ |
/// | `GenericType<()>` implements `SimpleTrait`                | ✅     | ✅      |
/// | `GenericType<!>` implements `GenericTrait<u32>`           | ❌     | ❌      |
/// | `GenericType<u32>` implements `GenericTrait<u32>`         | ✅     | ✅      |
/// | `GenericType<u32>` implements `TwoGenericTrait`           | ✅     | ❌      |
/// | `GenericType<u32>` implements `TwoGenericTrait<i32>`      | ✅     | ❌      |
/// | `GenericType<u32>` implements `TwoGenericTrait<i32, i32>` | ✅     | ✅      |
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Default)]
pub enum TraitTestMode {
    /// The comparison will enforce the given generic arguments and allow any
    /// additional ones to be freely matched. This is useful for traits with
    /// default types for generic parameters.
    ///
    /// See the documentation of [`TraitTestMode`] for a comparison of the
    /// different test modes.
    #[default]
    Lossy,
    /// The comparison will check that the correct number of generics has been
    /// provided and will check that all of them match.
    ///
    /// See the documentation of [`TraitTestMode`] for a comparison of the
    /// different test modes.
    Strict,
}

pub struct UserDefinedGeneric<'a> {
    _lifetime: PhantomData<&'a ()>,
}

#[derive(Debug)]
#[non_exhaustive]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
enum UserDefinedGenericInner<'a> {
    ApiType(TyKind<'a>),
    // TODO: Handle the path properly, since `Str` is not ABI safe
    Path(String),
}
