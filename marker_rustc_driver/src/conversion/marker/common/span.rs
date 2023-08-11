use marker_api::{ast::SpanPos, prelude::Span};

use crate::conversion::marker::MarkerConverterInner;

impl<'ast, 'tcx> MarkerConverterInner<'ast, 'tcx> {
    pub fn to_span(&self, rustc_span: rustc_span::Span) -> Span<'ast> {
        Span::new(
            self.to_span_src_id(rustc_span.ctxt()),
            // The driver resugars all expressions and spans, this should therefore
            // only be true for spans from macro expansion.
            rustc_span.from_expansion(),
            self.to_span_pos(rustc_span.lo()),
            self.to_span_pos(rustc_span.hi()),
        )
    }

    pub fn to_span_pos(&self, byte_pos: rustc_span::BytePos) -> SpanPos {
        SpanPos::new(byte_pos.0)
    }
}
