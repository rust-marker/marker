use std::marker::PhantomData;

use crate::ffi::FfiOption;

use super::{Span, SpanId};

// Primitive types
mod bool_ty;
pub use bool_ty::*;
mod text_ty;
pub use text_ty::*;
mod num_ty;
pub use num_ty::*;
mod never_ty;
pub use never_ty::*;
// Sequence types
mod tuple_ty;
pub use tuple_ty::*;
mod array_ty;
pub use array_ty::*;
mod slice_ty;
pub use slice_ty::*;
// User defined types
mod struct_ty;
pub use struct_ty::*;
mod enum_ty;
pub use enum_ty::*;
mod union_ty;
pub use union_ty::*;
// Function types
mod fn_ty;
pub use fn_ty::*;
mod closure_ty;
pub use closure_ty::*;
// Pointer types
mod ref_ty;
pub use ref_ty::*;
mod raw_ptr_ty;
pub use raw_ptr_ty::*;
mod fn_ptr_ty;
pub use fn_ptr_ty::*;
// Trait types
mod trait_obj_ty;
pub use trait_obj_ty::*;
mod impl_trait_ty;
pub use impl_trait_ty::*;
// Syntactic types
mod inferred_ty;
pub use inferred_ty::*;
mod generic_ty;
pub use generic_ty::*;
mod alias_ty;
pub use alias_ty::*;
mod self_ty;
pub use self_ty::*;
mod relative_ty;
pub use relative_ty::*;

pub trait TyData<'ast> {
    fn as_kind(&'ast self) -> TyKind<'ast>;

    /// The [`Span`] of the type, if it's written in the source code. Only
    /// syntactic types can have spans attached to them.
    ///
    /// Currently, every syntactic type will return a valid [`Span`] this can
    /// change in the future.
    fn span(&self) -> Option<&Span<'ast>>;

    // FIXME: I feel like `is_syntactic` and `is_semantic` are not the best
    // names to distinguish between the two sources, but I can't think of
    // something better, rn.

    /// Returns `true`, if this type instance originates from a written type.
    /// These types can have a [`Span`] attached to them.
    ///
    /// In the expression `let x: Vec<_> = vec![15];` the type declaration
    /// `: Vec<_>` syntactic type written by the user with a [`Span`]. The
    /// semantic type is `Vec<u32>`. Notice that the semantic type includes
    /// the inferred `u32` type in the vector. This separation also means, that
    /// a variable can have two different kinds, the written syntactic one and
    /// the semantic one.
    fn is_syntactic(&self) -> bool;

    /// Returns `true`, if this type instance is a semantic type, which was
    /// determined by the driver during translation.
    ///
    /// See [`is_syntactic`](`TyData::is_syntactic`) for an example.
    fn is_semantic(&self) -> bool;
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TyKind<'ast> {
    // ================================
    // Primitive types
    // ================================
    /// The `bool` type
    Bool(&'ast BoolTy<'ast>),
    /// A numeric type like `u32`, `i32`, `f64`
    Num(&'ast NumTy<'ast>),
    /// A textual type like `char` or `str`
    Text(&'ast TextTy<'ast>),
    /// The never type `!`
    Never(&'ast NeverTy<'ast>),
    // ================================
    // Sequence types
    // ================================
    /// A tuple type like `()`, `(T, U)`
    Tuple(&'ast TupleTy<'ast>),
    /// An array with a known size like: `[T; n]`
    Array(&'ast ArrayTy<'ast>),
    /// A variable length slice like `[T]`
    Slice(&'ast SliceTy<'ast>),
    // ================================
    // User define types
    // ================================
    Struct(&'ast StructTy<'ast>),
    Enum(&'ast EnumTy<'ast>),
    Union(&'ast UnionTy<'ast>),
    // ================================
    // Function types
    // ================================
    Fn(&'ast FnTy<'ast>),
    Closure(&'ast ClosureTy<'ast>),
    // ================================
    // Pointer types
    // ================================
    /// A reference like `&T` or `&mut T`
    Ref(&'ast RefTy<'ast>),
    /// A raw pointer like `*const T` or `*mut T`
    RawPtr(&'ast RawPtrTy<'ast>),
    /// A function pointer like `fn (T) -> U`
    FnPtr(&'ast FnPtrTy<'ast>),
    // ================================
    // Trait types
    // ================================
    /// A trait object like `dyn Trait`
    TraitObj(&'ast TraitObjTy<'ast>),
    /// An `impl Trait` type like:
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
    ImplTrait(&'ast ImplTraitTy<'ast>),
    // ================================
    // Syntactic type
    // ================================
    /// An inferred type
    Inferred(&'ast InferredTy<'ast>),
    /// A generic type, that has been specified in a surrounding item
    Generic(&'ast GenericTy<'ast>),
    Alias(&'ast AliasTy<'ast>),
    /// The `Self` in impl blocks or trait declarations
    SelfTy(&'ast SelfTy<'ast>),
    /// A type declared relative to another type, like `Iterator::Item`
    Relative(&'ast RelativeTy<'ast>),
}

impl<'ast> TyKind<'ast> {
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

    /// Returns `true` if this is a user defined type.
    #[must_use]
    pub fn is_user_defined_type(&self) -> bool {
        matches!(self, Self::Struct(..) | Self::Enum(..) | Self::Union(..))
    }

    /// Returns `true` if the ty kind is function type.
    #[must_use]
    pub fn is_fn(&self) -> bool {
        matches!(self, Self::Fn(..) | Self::Closure(..))
    }

    /// Returns `true` if this is a pointer type.
    #[must_use]
    pub fn is_ptr_ty(&self) -> bool {
        matches!(self, Self::Ref(..) | Self::RawPtr(..) | Self::FnPtr(..))
    }

    /// Returns `true` if the ty kind is trait type.
    #[must_use]
    pub fn is_trait_ty(&self) -> bool {
        matches!(self, Self::TraitObj(..) | Self::ImplTrait(..))
    }

    /// Returns `true` if the ty kind is syntactic type, meaning a type that is
    /// only used in syntax like [`TyKind::Inferred`] and [`TyKind::Generic`].
    ///
    /// See [`TyKind::is_syntactic()`] to check if this type originates from
    /// a syntactic definition.
    #[must_use]
    pub fn is_inferred(&self) -> bool {
        matches!(self, Self::Inferred(..))
    }
}

impl<'ast> TyKind<'ast> {
    impl_ty_data_fn!(span() -> Option<&Span<'ast>>);
    impl_ty_data_fn!(is_syntactic() -> bool);
    impl_ty_data_fn!(is_semantic() -> bool);
}

/// Until [trait upcasting](https://github.com/rust-lang/rust/issues/65991) has been implemented
/// and stabilized we need this to call [`TyData`] functions for every [`TyKind`].
macro_rules! impl_ty_data_fn {
    ($method:ident () -> $return_ty:ty) => {
        impl_ty_data_fn!($method() -> $return_ty,
        Bool, Num, Text, Never, Tuple, Array, Slice, Struct, Enum, Union, Fn,
        Closure, Ref, RawPtr, FnPtr, TraitObj, ImplTrait, Inferred, Generic,
        Alias, SelfTy, Relative);
    };
    ($method:ident () -> $return_ty:ty $(, $item:ident)+) => {
        pub fn $method(&self) -> $return_ty {
            match self {
                $(TyKind::$item(data) => data.$method(),)*
            }
        }
    };
}

use impl_ty_data_fn;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
pub(crate) struct CommonTyData<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    span: FfiOption<SpanId>,
    is_syntactic: bool,
}

#[cfg(feature = "driver-api")]
impl<'ast> CommonTyData<'ast> {
    pub fn new_syntactic(span: SpanId) -> Self {
        Self {
            _lifetime: PhantomData,
            span: Some(span).into(),
            is_syntactic: true,
        }
    }

    pub fn new_semantic() -> Self {
        Self {
            _lifetime: PhantomData,
            span: FfiOption::None,
            is_syntactic: false,
        }
    }
}

macro_rules! impl_ty_data {
    ($self_ty:ty, $enum_name:ident) => {
        impl<'ast> From<&'ast $self_ty> for $crate::ast::ty::TyKind<'ast> {
            fn from(from: &'ast $self_ty) -> Self {
                $crate::ast::ty::TyKind::$enum_name(from)
            }
        }

        impl<'ast> $crate::ast::ty::TyData<'ast> for $self_ty {
            fn as_kind(&'ast self) -> $crate::ast::ty::TyKind<'ast> {
                self.into()
            }

            fn span(&self) -> Option<&$crate::ast::Span<'ast>> {
                self.data
                    .span
                    .get()
                    .map(|span_id| $crate::context::with_cx(self, |cx| cx.get_span(*span_id)))
            }
            fn is_syntactic(&self) -> bool {
                self.data.is_syntactic
            }

            fn is_semantic(&self) -> bool {
                !self.data.is_syntactic
            }
        }
    };
}
use impl_ty_data;
