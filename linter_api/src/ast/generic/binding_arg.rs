use crate::{
    ast::{ty::TyKind, Span, SpanId, SymbolId},
    context::with_cx,
    ffi::FfiOption,
};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TyBinding<'ast> {
    span: FfiOption<SpanId>,
    ident: SymbolId,
    ty: TyKind<'ast>,
}

impl<'ast> TyBinding<'ast> {
    pub fn ident(&self) -> String {
        with_cx(self, |cx| cx.symbol_str(self.ident))
    }

    pub fn ty(&self) -> TyKind<'ast> {
        self.ty
    }

    pub fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| with_cx(self, |cx| cx.get_span(*span)))
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> TyBinding<'ast> {
    pub fn new(span: Option<SpanId>, ident: SymbolId, ty: TyKind<'ast>) -> Self {
        Self {
            span: span.into(),
            ident,
            ty,
        }
    }
}
