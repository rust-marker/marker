use crate::{
    ast::{AstPath, Span, SpanId, SymbolId, VarId},
    context::with_cx,
    ffi::{FfiOption, FfiSlice},
};

use super::{CommonPatData, PatKind};

#[repr(C)]
#[derive(Debug)]
pub struct StructPat<'ast> {
    data: CommonPatData<'ast>,
    path: AstPath<'ast>,
    fields: FfiSlice<'ast, StructFieldPat<'ast>>,
    is_exhaustive: bool,
}

impl<'ast> StructPat<'ast> {
    pub fn path(&self) -> &AstPath<'ast> {
        &self.path
    }

    pub fn fields(&self) -> &[StructFieldPat<'ast>] {
        self.fields.get()
    }

    /// Returns `true` if the pattern is non exhaustive. The pattern might still
    /// cover all fields, but have a rest pattern (`..`) to map for potential new
    /// fields or if the struct has been marked as `non_exhaustive`.
    pub fn is_non_exhaustive(&self) -> bool {
        self.is_exhaustive
    }
}

super::impl_pat_data!(StructPat<'ast>, Struct);

#[cfg(feature = "driver-api")]
impl<'ast> StructPat<'ast> {
    pub fn new(
        data: CommonPatData<'ast>,
        path: AstPath<'ast>,
        fields: &'ast [StructFieldPat<'ast>],
        is_exhaustive: bool,
    ) -> Self {
        Self {
            data,
            path,
            fields: fields.into(),
            is_exhaustive,
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct StructFieldPat<'ast> {
    span: SpanId,
    ident: SymbolId,
    var_id: FfiOption<VarId>,
    is_mut: bool,
    is_ref: bool,
    pat: FfiOption<PatKind<'ast>>,
}

impl<'ast> StructFieldPat<'ast> {
    pub fn span(&self) -> &Span<'ast> {
        with_cx(self, |cx| cx.get_span(self.span))
    }

    pub fn ident(&self) -> String {
        with_cx(self, |cx| cx.symbol_str(self.ident))
    }

    /// The [`VarId`] if this field pattern binds the field.
    pub fn var_id(&self) -> Option<VarId> {
        self.var_id.copy()
    }

    pub fn is_mut(&self) -> bool {
        self.is_mut
    }

    pub fn is_ref(&self) -> bool {
        self.is_ref
    }

    pub fn pat(&self) -> Option<PatKind<'ast>> {
        self.pat.copy()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> StructFieldPat<'ast> {
    pub fn new(
        span: SpanId,
        ident: SymbolId,
        var_id: Option<VarId>,
        is_mut: bool,
        is_ref: bool,
        pat: Option<PatKind<'ast>>,
    ) -> Self {
        Self {
            span,
            ident,
            var_id: var_id.into(),
            is_mut,
            is_ref,
            pat: pat.into(),
        }
    }
}
