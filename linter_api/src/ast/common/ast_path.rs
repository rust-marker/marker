//! This module contains all representations of paths in the AST.
//!
//! See: <https://doc.rust-lang.org/stable/reference/paths.html>

// FIXME: It might be useful to not use a single path for everything, but instead
// split it up into an `ItemPath`, `GenericPath` etc. implementation.

use std::marker::PhantomData;

use super::SymbolId;
use crate::{
    context::AstContext,
    ffi::{FfiOption, FfiSlice},
};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AstPath<'ast> {
    _cx: &'ast AstContext<'ast>,
    segments: FfiSlice<'ast, &'ast AstPathSegment<'ast>>,
}

impl<'ast> AstPath<'ast> {
    pub fn segments(&self) -> &[&AstPathSegment<'ast>] {
        self.segments.get()
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AstPathSegment<'ast> {
    cx: &'ast AstContext<'ast>,
    ident: SymbolId,
    generic_args: FfiOption<&'ast AstPathGenericArgs<'ast>>,
}

impl<'ast> AstPathSegment<'ast> {
    pub fn ident(&self) -> String {
        self.cx.symbol_str(self.ident)
    }

    pub fn generic_args(&self) -> Option<&AstPathGenericArgs<'ast>> {
        self.generic_args.get().copied()
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AstPathGenericArgs<'ast> {
    _phantom: PhantomData<&'ast ()>, // FIXME: Fill once lifetimes and types are completed.
}

#[cfg(feature = "driver-api")]
impl<'ast> AstPathGenericArgs<'ast> {
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { _phantom: PhantomData }
    }
}
