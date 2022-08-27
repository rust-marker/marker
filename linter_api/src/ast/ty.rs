use std::fmt::Debug;

use super::{CrateId, Lifetime, Mutability};

/// Rustc uses two different types, one for the IR and one for type resolution
/// and type checking. In this crate, we attempt to combine both into one trait
/// for simplicity. However, this also means that some functions are only meant
/// for one use-case. [`Ty::is_infered`] is an example for this as only type
/// annotations in code can have inferred types.
pub trait Ty<'ast>: Debug {
    fn get_kind(&'ast self) -> &'ast TyKind<'ast>;

    fn is_unit(&'ast self) -> bool {
        matches!(self.get_kind(), TyKind::Tuple(&[]))
    }

    /// In the expression `let v: Vec<_> = vec![1, 2, 3]`, the type would be
    /// `Vec<i32>` with `i32` being inferred by the usage.
    fn is_infered(&self) -> bool;
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TyId {
    krate: CrateId,
    index: u32,
}

#[cfg(feature = "driver-api")]
impl TyId {
    #[must_use]
    pub fn new(krate: CrateId, index: u32) -> Self {
        Self { krate, index }
    }

    pub fn get_data(&self) -> (CrateId, u32) {
        (self.krate, self.index)
    }
}

#[non_exhaustive]
#[derive(Clone, Debug)]
/// See: <https://doc.rust-lang.org/reference/types.html>
pub enum TyKind<'ast> {
    Bool,
    Numeric(NumericKind),
    Textual(TextualKind),
    Never,
    /// See: <https://doc.rust-lang.org/reference/types/tuple.html>
    Tuple(&'ast [&'ast dyn Ty<'ast>]),
    /// See: <https://doc.rust-lang.org/reference/types/array.html>
    ///
    /// FIXME: Add index expression
    Array(&'ast dyn Ty<'ast>),
    /// See: <https://doc.rust-lang.org/reference/types/slice.html>
    Slice(&'ast dyn Ty<'ast>),

    /// Algebraic data types (ADT) for instance structs, enums and unions.
    ///
    /// The inner type is linked with the given [`TyId`] this allows the representation
    /// of recursive data types, containing variations of themselves and simple type comparison
    /// using the [`TyId`]
    Adt(TyId),

    Ref(&'ast dyn Ty<'ast>, Mutability, &'ast dyn Lifetime<'ast>),

    RawPtr(&'ast dyn Ty<'ast>, Mutability),

    Fn(&'ast FunctionTy<'ast>),
    // FIXME Closure
    ImplTrait(TyId),

    DynTrait(TyId),

    /// A type alias like `type Foo = Bar`. The [`TyId`] belongs to the aliased
    /// type, in this case it would be the id of `Bar`.
    TyAlias(TyId),

    /// A type from an `extern` block
    ForeignTy(TyId),

    /// This value is used a type contains a type that is only available on nightly
    Unsupported,
}

/// See: <https://doc.rust-lang.org/reference/types/numeric.html>
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumericKind {
    Isize,
    I8,
    I16,
    I32,
    I64,
    I128,
    Usize,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
}

/// See: <https://doc.rust-lang.org/reference/types/textual.html>
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextualKind {
    Char,
    Str,
}

// This is an extra type as I expect that this can be expanded in the future
#[non_exhaustive]
#[derive(Debug)]
pub struct FunctionTy<'ast> {
    return_ty: Option<&'ast dyn Ty<'ast>>,

    args: &'ast [&'ast dyn Ty<'ast>],
}

#[cfg(feature = "driver-api")]
impl<'ast> FunctionTy<'ast> {
    #[must_use]
    pub fn new(return_ty: Option<&'ast dyn Ty<'ast>>, args: &'ast [&'ast dyn Ty<'ast>]) -> Self {
        Self { return_ty, args }
    }
}

impl<'ast> FunctionTy<'ast> {
    pub fn get_return_ty(&'ast self) -> Option<&'ast dyn Ty<'ast>> {
        self.return_ty
    }

    pub fn get_args(&'ast self) -> &'ast [&'ast dyn Ty<'ast>] {
        self.args
    }
}
