use crate::{
    ast::{ty::TyKind, Span, SpanId, SymbolId},
    context::AstContext,
    ffi::FfiOption,
};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TyBinding<'ast> {
    cx: &'ast AstContext<'ast>,
    span: FfiOption<SpanId>,
    ident: SymbolId,
    ty: TyKind<'ast>,
}

impl<'ast> TyBinding<'ast> {
    pub fn ident(&self) -> String {
        self.cx.symbol_str(self.ident)
    }

    pub fn ty(&self) -> TyKind<'ast> {
        self.ty
    }

    pub fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| self.cx.get_span(*span))
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> TyBinding<'ast> {
    pub fn new(cx: &'ast AstContext<'ast>, span: Option<SpanId>, ident: SymbolId, ty: TyKind<'ast>) -> Self {
        Self {
            cx,
            span: span.into(),
            ident,
            ty,
        }
    }
}
