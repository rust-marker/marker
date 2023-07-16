mod args;
pub use args::*;
mod param;
pub use param::*;

use crate::{ast::ty::SemTyKind, ffi::FfiSlice};

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
/// * [`SynGenericParams`][super::SynGenericParams]
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

/// A singular semantic generic argument.
///
/// See: <https://doc.rust-lang.org/stable/reference/paths.html>
#[repr(C)]
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(Clone))]
pub enum SemGenericArgKind<'ast> {
    /// A type as a generic argument, like this:
    ///
    /// ```
    /// let _bar: Vec<String> = vec!();
    /// //            ^^^^^^
    /// ```
    Ty(SemTyKind<'ast>),
    /// A type binding as a generic argument, like this:
    ///
    /// ```ignore
    /// let _baz: &dyn Iterator<Item=String> = todo!();
    /// //                      ^^^^^^^^^^^
    /// ```
    Binding(&'ast SemBindingArg<'ast>),
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
    Const(&'ast SemConstArg<'ast>),
}
