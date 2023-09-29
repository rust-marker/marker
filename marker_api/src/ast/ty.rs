use std::fmt::Debug;
use std::marker::PhantomData;

use crate::{private::Sealed, span::Span};

use super::SpanId;

// Primitive types
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

/// This trait combines methods, which are common between all syntactic types.
///
/// This trait is only meant to be implemented inside this crate. The `Sealed`
/// super trait prevents external implementations.
pub trait SynTyData<'ast>: Debug + Sealed {
    /// Returns `&self` wrapped in it's [`SynTyKind`] variant.
    ///
    /// In function parameters, it's recommended to use `Into<SynTyKind<'ast>>`
    /// as a bound to support all expressions and `SynTyKind<'ast>` as parameters.
    fn as_kind(&'ast self) -> SynTyKind<'ast>;

    /// The [`Span`] of the type, if it's written in the source code.
    fn span(&self) -> &Span<'ast>;
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum SynTyKind<'ast> {
    // ================================
    // Primitive types
    // ================================
    /// The `bool` type
    Bool(&'ast SynBoolTy<'ast>),
    /// A numeric type like [`u32`], [`i32`], [`f64`]
    Num(&'ast SynNumTy<'ast>),
    /// A textual type like [`char`] or [`str`]
    Text(&'ast SynTextTy<'ast>),
    /// The never type [`!`](prim@never)
    Never(&'ast SynNeverTy<'ast>),
    // ================================
    // Sequence types
    // ================================
    /// A tuple type like [`()`](prim@tuple), [`(T, U)`](prim@tuple)
    Tuple(&'ast SynTupleTy<'ast>),
    /// An array with a known size like: [`[T; N]`](prim@array)
    Array(&'ast SynArrayTy<'ast>),
    /// A variable length slice like [`[T]`](prim@slice)
    Slice(&'ast SynSliceTy<'ast>),
    // ================================
    // Pointer types
    // ================================
    /// A reference like [`&T`](prim@reference) or [`&mut T`](prim@reference)
    Ref(&'ast SynRefTy<'ast>),
    /// A raw pointer like [`*const T`](prim@pointer) or [`*mut T`](prim@pointer)
    RawPtr(&'ast SynRawPtrTy<'ast>),
    /// A function pointer like [`fn (T) -> U`](prim@fn)
    FnPtr(&'ast SynFnPtrTy<'ast>),
    // ================================
    // Trait types
    // ================================
    /// A trait object like [`dyn Trait`](https://doc.rust-lang.org/stable/std/keyword.dyn.html)
    TraitObj(&'ast SynTraitObjTy<'ast>),
    /// An [`impl Trait`](https://doc.rust-lang.org/stable/std/keyword.impl.html) type like:
    ///
    /// ```
    /// trait Trait {}
    /// # impl Trait for () {}
    ///
    /// // argument position: anonymous type parameter
    /// fn foo(arg: impl Trait) {
    /// }
    ///
    /// // return position: abstract return type
    /// fn bar() -> impl Trait {
    /// }
    /// ```
    ///
    /// See: <https://doc.rust-lang.org/stable/reference/types/impl-trait.html>
    ImplTrait(&'ast SynImplTraitTy<'ast>),
    // ================================
    // Syntactic types
    // ================================
    /// An inferred type
    Inferred(&'ast SynInferredTy<'ast>),
    Path(&'ast SynPathTy<'ast>),
}

impl<'ast> SynTyKind<'ast> {
    /// Returns `true` if this is a primitive type.
    #[must_use]
    pub fn is_primitive_ty(&self) -> bool {
        matches!(self, Self::Bool(..) | Self::Num(..) | Self::Text(..) | Self::Never(..))
    }

    /// Returns `true` if this is a sequence type.
    #[must_use]
    pub fn is_sequence_ty(&self) -> bool {
        matches!(self, Self::Tuple(..) | Self::Array(..) | Self::Slice(..))
    }

    /// Returns `true` if this is a function type.
    #[must_use]
    pub fn is_fn(&self) -> bool {
        matches!(self, Self::FnPtr(..))
    }

    /// Returns `true` if this is a pointer type.
    #[must_use]
    pub fn is_ptr_ty(&self) -> bool {
        matches!(self, Self::Ref(..) | Self::RawPtr(..) | Self::FnPtr(..))
    }

    /// Returns `true` if this is a trait type.
    #[must_use]
    pub fn is_trait_ty(&self) -> bool {
        matches!(self, Self::TraitObj(..) | Self::ImplTrait(..))
    }

    #[must_use]
    pub fn is_inferred(&self) -> bool {
        matches!(self, Self::Inferred(..))
    }
}

impl<'ast> SynTyKind<'ast> {
    impl_syn_ty_data_fn!(span() -> &Span<'ast>);
}

/// Until [trait upcasting](https://github.com/rust-lang/rust/issues/65991) has been implemented
/// and stabilized we need this to call [`SynTyData`] functions for every [`SynTyKind`].
macro_rules! impl_syn_ty_data_fn {
    ($method:ident () -> $return_ty:ty) => {
        impl_syn_ty_data_fn!($method() -> $return_ty,
            Bool, Num, Text, Never,
            Tuple, Array, Slice,
            Ref, RawPtr, FnPtr,
            TraitObj, ImplTrait,
            Inferred, Path
        );
    };
    ($method:ident () -> $return_ty:ty $(, $item:ident)+) => {
        pub fn $method(&self) -> $return_ty {
            match self {
                $(SynTyKind::$item(data) => data.$method(),)*
            }
        }
    };
}

use impl_syn_ty_data_fn;

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
pub(crate) struct CommonSynTyData<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    span: SpanId,
}

#[cfg(feature = "driver-api")]
impl<'ast> CommonSynTyData<'ast> {
    pub fn new_syntactic(span: SpanId) -> Self {
        Self {
            _lifetime: PhantomData,
            span,
        }
    }
}

macro_rules! impl_ty_data {
    ($self_ty:ty, $enum_name:ident) => {
        impl<'ast> $crate::ast::ty::SynTyData<'ast> for $self_ty {
            fn as_kind(&'ast self) -> $crate::ast::ty::SynTyKind<'ast> {
                self.into()
            }

            fn span(&self) -> &$crate::span::Span<'ast> {
                $crate::context::with_cx(self, |cx| cx.span(self.data.span))
            }
        }

        impl<'ast> $crate::private::Sealed for $self_ty {}

        impl<'ast> From<&'ast $self_ty> for $crate::ast::ty::SynTyKind<'ast> {
            fn from(from: &'ast $self_ty) -> Self {
                $crate::ast::ty::SynTyKind::$enum_name(from)
            }
        }
    };
}
use impl_ty_data;

/// The semantic representation of a type.
#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum SemTyKind<'ast> {
    // ================================
    // Primitive types
    // ================================
    /// The `bool` type
    Bool(&'ast SemBoolTy<'ast>),
    /// A numeric type like [`u32`], [`i32`], [`f64`]
    Num(&'ast SemNumTy<'ast>),
    /// A textual type like [`char`] or [`str`]
    Text(&'ast SemTextTy<'ast>),
    /// The never type [`!`](prim@never)
    Never(&'ast SemNeverTy<'ast>),
    // ================================
    // Sequence types
    // ================================
    /// A tuple type like [`()`](prim@tuple), [`(T, U)`](prim@tuple)
    Tuple(&'ast SemTupleTy<'ast>),
    /// An array with a known size like: [`[T; N]`](prim@array)
    Array(&'ast SemArrayTy<'ast>),
    /// A variable length slice like [`[T]`](prim@slice)
    Slice(&'ast SemSliceTy<'ast>),
    // ================================
    // Function types
    // ================================
    /// A [function item type](https://doc.rust-lang.org/reference/types/function-item.html)
    /// identifying a specific function and potentualy additional generics.
    FnTy(&'ast SemFnTy<'ast>),
    /// The semantic representation of a
    /// [closure type](https://doc.rust-lang.org/reference/types/closure.html).
    ClosureTy(&'ast SemClosureTy<'ast>),
    // ================================
    // Pointer types
    // ================================
    /// A reference like [`&T`](prim@reference) or [`&mut T`](prim@reference)
    Ref(&'ast SemRefTy<'ast>),
    /// A raw pointer like [`*const T`](prim@pointer) or [`*mut T`](prim@pointer)
    RawPtr(&'ast SemRawPtrTy<'ast>),
    /// The semantic representation of a function pointer, like [`fn (T) -> U`](prim@fn)
    FnPtr(&'ast SemFnPtrTy<'ast>),
    // ================================
    // Trait types
    // ================================
    /// A trait object like [`dyn Trait`](https://doc.rust-lang.org/stable/std/keyword.dyn.html)
    TraitObj(&'ast SemTraitObjTy<'ast>),
    // ================================
    // User defined types
    // ================================
    /// A user defined data type, identified by an [`TyDefId`](super::TyDefId)
    Adt(&'ast SemAdtTy<'ast>),
    /// A generic type defined by a generic parameter
    Generic(&'ast SemGenericTy<'ast>),
    /// A type alias. Note that simple type aliases will already be replaced in
    /// semantic types. This kind is mainly used for type aliases, where the concrete
    /// type is not yet known, for example in traits.
    Alias(&'ast SemAliasTy<'ast>),
    // ================================
    // Other types
    // ================================
    /// The placeholder type, signalling that the semantic type is still unstable
    /// and therefor not represented as part of the API.
    Unstable(&'ast SemUnstableTy<'ast>),
}
