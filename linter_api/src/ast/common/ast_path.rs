//! This module contains all representations of paths in the AST.
//!
//! See: <https://doc.rust-lang.org/stable/reference/paths.html>

// FIXME: It might be useful to not use a single path for everything, but instead
// split it up into an `ItemPath`, `GenericPath` etc. implementation.

use super::SymbolId;
use crate::{
    ast::generic::GenericArgs,
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
    generic_args: FfiOption<&'ast GenericArgs<'ast>>,
}

#[cfg(feature = "driver-api")]
impl<'ast> AstPathSegment<'ast> {
    pub fn new(cx: &'ast AstContext<'ast>, ident: SymbolId, generic_args: Option<&'ast GenericArgs<'ast>>) -> Self {
        Self {
            cx,
            ident,
            generic_args: generic_args.into(),
        }
    }
}

impl<'ast> AstPathSegment<'ast> {
    pub fn ident(&self) -> String {
        self.cx.symbol_str(self.ident)
    }

    pub fn generic_args(&self) -> Option<&GenericArgs<'ast>> {
        self.generic_args.get().copied()
    }
}
