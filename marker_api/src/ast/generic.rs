use std::fmt::Debug;

use super::ty::{SemTy, TyKind};
use crate::ffi::FfiSlice;

mod arg;
pub use arg::*;
mod bound;
pub use bound::*;
mod param;
pub use param::*;
mod clause;
pub use clause::*;

/// This represents the generic arguments for an item.
///
/// ```
/// # use std::fmt::Debug;
/// //             vv This is a generic argument
/// generic_item::<u8>(32);
///
/// pub fn generic_item<T: Copy>(t: T)
/// //                  ^^^^^^^ This is a generic parameter
/// where
///     T: Debug,
/// //  ^^^^^^^^ This is a bound for a generic parameter
/// {
///     println!("{:#?}", t);
/// }
/// ```
///
/// See:
/// * [`GenericParams`]
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "driver-api", derive(Clone))]
pub struct GenericArgs<'ast> {
    args: FfiSlice<'ast, GenericArgKind<'ast>>,
}

impl<'ast> GenericArgs<'ast> {
    pub fn args(&self) -> &'ast [GenericArgKind<'ast>] {
        self.args.get()
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> GenericArgs<'ast> {
    pub fn new(args: &'ast [GenericArgKind<'ast>]) -> Self {
        Self { args: args.into() }
    }
}

/// A singular generic argument.
///
/// See: <https://doc.rust-lang.org/stable/reference/paths.html>
#[repr(C)]
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "driver-api", derive(Clone))]
pub enum GenericArgKind<'ast> {
    /// A lifetime as a generic argument, like this:
    ///
    /// ```
    /// # use std::marker::PhantomData;
    /// # #[derive(Default)]
    /// # pub struct HasLifetime<'a> {
    /// #     _data: PhantomData<&'a ()>,
    /// # }
    /// let _foo: HasLifetime<'static> = HasLifetime::default();
    /// //                    ^^^^^^^
    /// ```
    Lifetime(&'ast Lifetime<'ast>),
    /// A type as a generic argument, like this:
    ///
    /// ```
    /// let _bar: Vec<String> = vec!();
    /// //            ^^^^^^
    /// ```
    Ty(&'ast TyKind<'ast>),
    /// A type binding as a generic argument, like this:
    ///
    /// ```ignore
    /// let _baz: &dyn Iterator<Item=String> = todo!();
    /// //                      ^^^^^^^^^^^
    /// ```
    Binding(&'ast BindingGenericArg<'ast>),
    // FIXME: Add GenericArgsConst
}

/// This represents the generic arguments for an item.
///
/// ```
/// # use std::fmt::Debug;
/// //             vv This is a generic argument
/// generic_item::<u8>(32);
///
/// pub fn generic_item<T: Copy>(t: T)
/// //                  ^^^^^^^ This is a generic parameter
/// where
///     T: Debug,
/// //  ^^^^^^^^ This is a bound for a generic parameter
/// {
///     println!("{:#?}", t);
/// }
/// ```
///
/// See:
/// * [`GenericParams`]
#[repr(C)]
#[derive(Debug)]
pub struct SemGenericArgs<'ast> {
    args: FfiSlice<'ast, SemGenericArgKind<'ast>>,
}

impl<'ast> SemGenericArgs<'ast> {
    pub fn args(&self) -> &[SemGenericArgKind<'ast>] {
        self.args.get()
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SemGenericArgs<'ast> {
    pub fn new(args: &'ast [SemGenericArgKind<'ast>]) -> Self {
        Self { args: args.into() }
    }
}

/// A singular generic argument.
///
/// See: <https://doc.rust-lang.org/stable/reference/paths.html>
#[repr(C)]
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(Clone))]
pub enum SemGenericArgKind<'ast> {
    /// A lifetime as a generic argument, like this:
    ///
    /// ```
    /// # use std::marker::PhantomData;
    /// # #[derive(Default)]
    /// # pub struct HasLifetime<'a> {
    /// #     _data: PhantomData<&'a ()>,
    /// # }
    /// let _foo: HasLifetime<'static> = HasLifetime::default();
    /// //                    ^^^^^^^
    /// ```
    Lifetime(&'ast SemLifetime<'ast>),
    /// A type as a generic argument, like this:
    ///
    /// ```
    /// let _bar: Vec<String> = vec!();
    /// //            ^^^^^^
    /// ```
    Ty(&'ast SemTy<'ast>),
    /// A type binding as a generic argument, like this:
    ///
    /// ```ignore
    /// let _baz: &dyn Iterator<Item=String> = todo!();
    /// //                      ^^^^^^^^^^^
    /// ```
    TyBinding(&'ast SemTyBindingArg<'ast>),
    // FIXME: Add GenericArgsConst
}

/// This represents the generic parameters of a generic item. The bounds applied
/// to the parameters in the declaration are stored as clauses in this struct.
///
/// ```
/// # use std::fmt::Debug;
/// pub fn generic_item<T: Copy>(t: T)
/// //                  ^^^^^^^ This is a generic parameter
/// where
///     T: Debug,
/// //  ^^^^^^^^  This is a bound for a generic parameter
/// {
///     println!("{:#?}", t);
/// }
///
/// //             vv This is a generic argument
/// generic_item::<u8>(32);
/// ```
/// See
/// * [`GenericArgs`]
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct GenericParams<'ast> {
    params: FfiSlice<'ast, GenericParamKind<'ast>>,
    clauses: FfiSlice<'ast, WhereClauseKind<'ast>>,
}

impl<'ast> GenericParams<'ast> {
    pub fn params(&self) -> &'ast [GenericParamKind<'ast>] {
        self.params.get()
    }

    pub fn clauses(&self) -> &'ast [WhereClauseKind<'ast>] {
        self.clauses.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> GenericParams<'ast> {
    pub fn new(params: &'ast [GenericParamKind<'ast>], clauses: &'ast [WhereClauseKind<'ast>]) -> Self {
        Self {
            params: params.into(),
            clauses: clauses.into(),
        }
    }
}
