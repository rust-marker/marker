use crate::{
    ast::{AstQPath, Span, SpanId, SymbolId},
    context::with_cx,
    ffi::FfiSlice,
};

use super::{CommonPatData, PatKind};

#[repr(C)]
#[derive(Debug)]
pub struct StructPat<'ast> {
    data: CommonPatData<'ast>,
    path: AstQPath<'ast>,
    fields: FfiSlice<'ast, StructFieldPat<'ast>>,
    is_non_exhaustive: bool,
}

impl<'ast> StructPat<'ast> {
    pub fn path(&self) -> &AstQPath<'ast> {
        &self.path
    }

    pub fn fields(&self) -> &[StructFieldPat<'ast>] {
        self.fields.get()
    }

    /// Returns `true` if the pattern is non exhaustive. The pattern might still
    /// cover all fields, but have a rest pattern (`..`) to map for potential new
    /// fields or if the struct has been marked as `non_exhaustive`.
    pub fn is_non_exhaustive(&self) -> bool {
        self.is_non_exhaustive
    }
}

super::impl_pat_data!(StructPat<'ast>, Struct);

#[cfg(feature = "driver-api")]
impl<'ast> StructPat<'ast> {
    pub fn new(
        data: CommonPatData<'ast>,
        path: AstQPath<'ast>,
        fields: &'ast [StructFieldPat<'ast>],
        is_non_exhaustive: bool,
    ) -> Self {
        Self {
            data,
            path,
            fields: fields.into(),
            is_non_exhaustive,
        }
    }
}

/// A pattern matching a field inside a [`StructPat`] instance.
///
/// Patterns that bind values directly to fields like `Struct {ref mut x}` are
/// expressed as `Struct {x: ref mut x}`. This allows a uniform view of all field
/// patterns. (This representation was inspired by rustc)
#[repr(C)]
#[derive(Debug)]
pub struct StructFieldPat<'ast> {
    span: SpanId,
    ident: SymbolId,
    pat: PatKind<'ast>,
}

impl<'ast> StructFieldPat<'ast> {
    pub fn span(&self) -> &Span<'ast> {
        with_cx(self, |cx| cx.get_span(self.span))
    }

    pub fn ident(&self) -> &str {
        with_cx(self, |cx| cx.symbol_str(self.ident))
    }

    pub fn pat(&self) -> PatKind<'ast> {
        self.pat
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> StructFieldPat<'ast> {
    pub fn new(span: SpanId, ident: SymbolId, pat: PatKind<'ast>) -> Self {
        Self { span, ident, pat }
    }
}
