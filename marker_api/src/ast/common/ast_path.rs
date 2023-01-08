//! This module contains all representations of paths in the AST.
//!
//! See: <https://doc.rust-lang.org/stable/reference/paths.html>

// FIXME: It might be useful to not use a single path for everything, but instead
// split it up into an `ItemPath`, `GenericPath` etc. implementation.

use super::SymbolId;
use crate::{
    ast::generic::GenericArgs,
    context::with_cx,
    ffi::{FfiOption, FfiSlice},
};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AstPath<'ast> {
    // FIXME: Add optional target ID for values, lifetimes, etc that is faster to compare
    //
    // You were last trying to fix the compiler error related to lifetime identification and paths
    segments: FfiSlice<'ast, AstPathSegment<'ast>>,
}

#[cfg(feature = "driver-api")]
impl<'ast> AstPath<'ast> {
    pub fn new(segments: &'ast [AstPathSegment<'ast>]) -> Self {
        Self {
            segments: segments.into(),
        }
    }
}

impl<'ast> AstPath<'ast> {
    pub fn segments(&self) -> &[AstPathSegment<'ast>] {
        self.segments.get()
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AstPathSegment<'ast> {
    ident: SymbolId,
    generics: FfiOption<GenericArgs<'ast>>,
}

#[cfg(feature = "driver-api")]
impl<'ast> AstPathSegment<'ast> {
    pub fn new(ident: SymbolId, generics: Option<GenericArgs<'ast>>) -> Self {
        Self {
            ident,
            generics: generics.into(),
        }
    }
}

impl<'ast> AstPathSegment<'ast> {
    pub fn ident(&self) -> String {
        with_cx(self, |cx| cx.symbol_str(self.ident))
    }

    pub fn generics(&self) -> Option<&GenericArgs<'ast>> {
        self.generics.get()
    }
}
