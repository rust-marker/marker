//! FIXME: Should this module remain under `ast::item::` or be moved to `ast::common`

use std::fmt::Debug;

use crate::{context::AstContext, ffi::FfiSlice};

mod lifetime_param;
pub use lifetime_param::*;
mod type_param;
pub use type_param::*;
mod binding_arg;
pub use binding_arg::*;

use super::{ty::TyKind, Span};

/// This represents the generic arguments for an item.
///
/// ```
/// # use std::fmt::Debug;
/// //            vvvv This is a generic argument
/// generic_item::<u8>(32);
///
/// pub fn generic_item<T: Copy>(t: T)
/// //                  ^^^^^^^ This is a generic parameter
/// where
///     T: Debug,
/// //  ^^^^^^^^ This is a second generic parameter
/// {
///     println!("{:#?}", t);
/// }
/// ```
///
/// See
/// * [`GenericParams`]
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct GenericArgs<'ast> {
    _cx: &'ast AstContext<'ast>,
    args: FfiSlice<'ast, GenericArg<'ast>>,
}

impl<'ast> GenericArgs<'ast> {
    pub fn new(cx: &'ast AstContext<'ast>, args: &'ast [GenericArg<'ast>]) -> Self {
        Self {
            _cx: cx,
            args: args.into(),
        }
    }
}

/// A singular generic argument.
///
/// See: <https://doc.rust-lang.org/stable/reference/paths.html>
#[repr(C)]
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum GenericArg<'ast> {
    Lifetime(&'ast Lifetime<'ast>),
    Type(&'ast TyKind<'ast>),
    Binding(&'ast TyBinding<'ast>),
    // FIXME: Add GenericArgsConst | GenericArgsBinding
}

/// This represents the generic parameters of a generic item.
///
/// ```
/// # use std::fmt::Debug;
/// pub fn generic_item<T: Copy>(t: T)
/// //                  ^^^^^^^ This is a generic parameter
/// where
///     T: Debug,
/// //  ^^^^^^^^ This is a second generic parameter
/// {
///     println!("{:#?}", t);
/// }
///
/// //            vvvv This is a generic argument
/// generic_item::<u8>(32);
/// ```
/// /// See
/// * [`GenericArgs`]
#[repr(C)]
#[derive(Debug)]
pub struct GenericParams<'ast> {
    _cx: &'ast AstContext<'ast>,
    params: FfiSlice<'ast, GenericParamKind<'ast>>,
}

#[cfg(feature = "driver-api")]
impl<'ast> GenericParams<'ast> {
    #[allow(clippy::used_underscore_binding)]
    pub fn new(_cx: &'ast AstContext<'ast>, params: &'ast [GenericParamKind<'ast>]) -> Self {
        Self {
            _cx,
            params: params.into(),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
#[non_exhaustive]
pub enum GenericParamKind<'ast> {
    Lifetime(&'ast LifetimeParam<'ast>),
    Type(&'ast TypeParam<'ast>), // FIXME: Add const `ConstParam`
}

/// This combines common
pub trait GenericParamData<'ast>: Debug {
    fn span(&self) -> Option<&Span<'ast>>;
    // FIXME: Add `fn attrs(&self) -> &[&Attrs<'ast>]` once implemented.
}
