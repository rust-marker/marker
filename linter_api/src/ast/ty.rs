use crate::context::AstContext;

use super::{Span, SpanId};

// Primitive types
mod boolean_ty;
pub use boolean_ty::*;
mod textual_ty;
pub use textual_ty::*;
mod numeric_ty;
pub use numeric_ty::*;
mod never_ty;
pub use never_ty::*;
// Sequence types
mod tuple_ty;
pub use tuple_ty::*;
mod array_ty;
pub use array_ty::*;
mod slice_ty;
pub use slice_ty::*;
// Pointer types
mod ref_ty;
pub use ref_ty::*;
mod raw_ptr_ty;
pub use raw_ptr_ty::*;

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
    /// See [`is_syntactic`][`TyData::is_syntactic`] for an example.
    fn is_semantic(&self) -> bool;
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum TyKind<'ast> {
    // ================================
    // Primitive types
    // ================================
    /// The `bool` type
    Boolean(&'ast BooleanTy<'ast>),
    /// A numeric type like `u32`, `i32`, `f64`
    Numeric(&'ast NumericTy<'ast>),
    /// A textual type like `char` or `str`
    Textual(&'ast TextualTy<'ast>),
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
    Struct, // (&'ast StructTy<'ast>),
    Enum,   // (&'ast EnumTy<'ast>),
    Union,  // (&'ast UnionTy<'ast>),
    // ================================
    // Function types
    // ================================
    Function, // (&'ast FunctionTy<'ast>),
    Closure,  // (&'ast ClosureTy<'ast>),
    // ================================
    // Pointer types
    // ================================
    /// A reference like `&T` or `&mut T`
    Ref(&'ast RefTy<'ast>),
    /// A raw pointer like `*const T` or `*mut T`
    RawPtr(&'ast RawPtrTy<'ast>),
    /// A function pointer like `fn (T) -> U`
    FunctionPtr, // (&'ast FunctionPointerTy<'ast>),
    // ================================
    // Trait types
    // ================================
    TraitObject, // (&'ast TraitObjectTy<'ast>),
    ImplTrait,   // (&'ast ImplTraitTy<'ast>),
    // ================================
    // Syntactic type
    // ================================
    Inferred, // (&'ast InferredTy<'ast>),
}
// FIXME: Do we want to keep the abbreviated pointer type names?

impl<'ast> TyKind<'ast> {
    /// Returns `true` if the ty kind is [`Boolean`].
    ///
    /// [`Boolean`]: TyKind::Boolean
    #[must_use]
    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Boolean(..))
    }

    /// Returns `true` if the ty kind is [`Numeric`].
    ///
    /// [`Numeric`]: TyKind::Numeric
    #[must_use]
    pub fn is_numeric(&self) -> bool {
        matches!(self, Self::Numeric(..))
    }

    /// Returns `true` if the ty kind is [`Textual`].
    ///
    /// [`Textual`]: TyKind::Textual
    #[must_use]
    pub fn is_textual(&self) -> bool {
        matches!(self, Self::Textual(..))
    }

    /// Returns `true` if the ty kind is [`Never`].
    ///
    /// [`Never`]: TyKind::Never
    #[must_use]
    pub fn is_never(&self) -> bool {
        matches!(self, Self::Never(..))
    }

    /// Returns `true` if this is a primitive type.
    #[must_use]
    pub fn is_primitive_ty(&self) -> bool {
        matches!(
            self,
            Self::Boolean(..) | Self::Numeric(..) | Self::Textual(..) | Self::Never(..)
        )
    }

    /// Returns `true` if the ty kind is [`Tuple`].
    ///
    /// [`Tuple`]: TyKind::Tuple
    #[must_use]
    pub fn is_tuple(&self) -> bool {
        matches!(self, Self::Tuple(..))
    }

    /// Returns `true` if the ty kind is [`Array`].
    ///
    /// [`Array`]: TyKind::Array
    #[must_use]
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(..))
    }

    /// Returns `true` if the ty kind is [`Slice`].
    ///
    /// [`Slice`]: TyKind::Slice
    #[must_use]
    pub fn is_slice(&self) -> bool {
        matches!(self, Self::Slice(..))
    }

    /// Returns `true` if this is a sequence type.
    #[must_use]
    pub fn is_sequence_ty(&self) -> bool {
        matches!(self, Self::Tuple(..) | Self::Array(..) | Self::Slice(..))
    }
}

#[repr(C)]
#[derive(Debug)]
#[non_exhaustive]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
pub(crate) struct CommonTyData<'ast> {
    cx: &'ast AstContext<'ast>,
    span: Option<SpanId>,
    is_syntactic: bool,
}

#[cfg(feature = "driver-api")]
impl<'ast> CommonTyData<'ast> {
    pub fn new_syntactic(cx: &'ast AstContext<'ast>, span: SpanId) -> Self {
        Self {
            cx,
            span: Some(span),
            is_syntactic: true,
        }
    }

    pub fn new_semantic(cx: &'ast AstContext<'ast>) -> Self {
        Self {
            cx,
            span: None,
            is_syntactic: false,
        }
    }
}

macro_rules! impl_ty_data {
    ($self_ty:ty, $enum_name:ident) => {
        impl<'ast> $crate::ast::ty::TyData<'ast> for $self_ty {
            fn as_kind(&'ast self) -> $crate::ast::ty::TyKind<'ast> {
                $crate::ast::ty::TyKind::$enum_name(self)
            }

            fn span(&self) -> Option<&$crate::ast::Span<'ast>> {
                self.data
                    .span
                    .map(|span_id| self.data.cx.get_span(&span_id.into()))
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
