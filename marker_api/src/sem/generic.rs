mod args;
mod param;
pub use args::*;
pub use param::*;

use crate::{ffi::FfiSlice, sem::ty::TyKind};

/// The semantic representation of generic arguments for an item or path.
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
/// * [`GenericParams`][crate::ast::generic::GenericParams]
#[repr(C)]
#[derive(Debug)]
pub struct GenericArgs<'ast> {
    args: FfiSlice<'ast, GenericArgKind<'ast>>,
}

impl<'ast> GenericArgs<'ast> {
    pub fn args(&self) -> &[GenericArgKind<'ast>] {
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

/// A singular semantic generic argument.
///
/// See: <https://doc.rust-lang.org/stable/reference/paths.html>
#[repr(C)]
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(Clone))]
pub enum GenericArgKind<'ast> {
    /// A type as a generic argument, like this:
    ///
    /// ```
    /// let _bar: Vec<String> = vec!();
    /// //            ^^^^^^
    /// ```
    Ty(TyKind<'ast>),
    /// A type binding as a generic argument, like this:
    ///
    /// ```ignore
    /// let _baz: &dyn Iterator<Item=String> = todo!();
    /// //                      ^^^^^^^^^^^
    /// ```
    Binding(&'ast BindingArg<'ast>),
    /// A constant expression as a generic argument, like this:
    ///
    /// ```ignore
    /// # struct Vec<const N: usize> {
    /// #     data: [f32; N],
    /// # }
    /// #
    /// let _bat: Vec<3> = todo!();
    /// //            ^
    /// ```
    Const(&'ast ConstArg<'ast>),
}
