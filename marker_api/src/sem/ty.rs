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
    /// A user defined data type, identified by an [`TyDefId`](marker_api::ast::TyDefId)
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
