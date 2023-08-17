use marker_api::{
    ast::{ExpnInfo, FileInfo, SpanPos, SpanSource},
    prelude::Span,
};

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

    pub fn to_span_source(&self, rust_span: rustc_span::Span) -> SpanSource<'ast> {
        let ctxt = rust_span.ctxt();
        if ctxt.is_root() {
            let src_file = self.rustc_cx.sess.source_map().lookup_source_file(rust_span.lo());
            let name = match &src_file.name {
                rustc_span::FileName::Real(
                    rustc_span::RealFileName::LocalPath(file_path)
                    | rustc_span::RealFileName::Remapped {
                        virtual_name: file_path,
                        ..
                    },
                ) => file_path.to_string_lossy().to_string(),
                _ => {
                    format!("MarkerConverter::to_span_source(): Unexpected file name: {rust_span:#?} -> {src_file:#?}")
                },
            };
            SpanSource::File(self.alloc(FileInfo::new(self.storage.alloc_str(&name), self.to_span_src_id(ctxt))))
        } else {
            SpanSource::Macro(self.alloc(self.to_expn_info(&ctxt.outer_expn_data())))
        }
    }

    pub fn try_to_expn_info(&self, id: rustc_span::ExpnId) -> Option<&'ast ExpnInfo<'ast>> {
        (id != rustc_span::ExpnId::root()).then(|| self.alloc(self.to_expn_info(&id.expn_data())))
    }

    pub fn to_expn_info(&self, data: &rustc_span::ExpnData) -> ExpnInfo<'ast> {
        debug_assert!(
            matches!(&data.kind, rustc_span::ExpnKind::Macro(_, _)),
            "this expansion data doesn't belong to a macro: {data:#?}"
        );
        ExpnInfo::new(
            self.to_expn_id(data.parent),
            self.to_span_id(data.call_site),
            self.to_macro_id(data.macro_def_id.expect("filled, because this belongs to a macro")),
        )
    }
}
