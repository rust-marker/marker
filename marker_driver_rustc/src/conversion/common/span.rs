use marker_api::ast::Span;
use marker_api::ast::SpanSource;
use rustc_span::BytePos;

use crate::context::RustcContext;

pub fn to_api_span<'ast, 'tcx>(cx: &'ast RustcContext<'ast, 'tcx>, rustc_span: rustc_span::Span) -> &'ast Span<'ast> {
    let (src, src_info) = to_api_src_info(cx, rustc_span);
    let start = (rustc_span.lo().0 as usize) - src_info.rustc_start_offset;
    let end = (rustc_span.hi().0 as usize) - src_info.rustc_start_offset;
    cx.storage.alloc(|| Span::new(src, start, end))
}

fn to_api_src_info<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    rustc_span: rustc_span::Span,
) -> (SpanSource<'ast>, SpanSourceInfo) {
    let map = cx.rustc_cx.sess.source_map();
    let rustc_src = map.lookup_source_file(rustc_span.lo());

    if let Some(api_src) = cx.storage.span_src(&rustc_src.name) {
        if let Some(src_info) = cx.storage.span_src_info(api_src) {
            return (api_src, src_info);
        }
        unreachable!("each `SpanSource` object should also have a `SpanSourceInfo` object")
    }

    let api_src = match &rustc_src.name {
        rustc_span::FileName::Real(real_name) => match real_name {
            rustc_span::RealFileName::LocalPath(path)
            | rustc_span::RealFileName::Remapped { virtual_name: path, .. } => {
                SpanSource::File(cx.storage.alloc(|| path.clone()))
            },
        },
        rustc_span::FileName::MacroExpansion(_) => todo!(),
        rustc_span::FileName::ProcMacroSourceCode(_) => todo!(),
        rustc_span::FileName::QuoteExpansion(_)
        | rustc_span::FileName::Anon(_)
        | rustc_span::FileName::CfgSpec(_)
        | rustc_span::FileName::CliCrateAttr(_)
        | rustc_span::FileName::Custom(_)
        | rustc_span::FileName::DocTest(_, _)
        | rustc_span::FileName::InlineAsm(_) => {
            unimplemented!("the api should only receive an request spans from files and macros")
        },
    };
    let api_info = SpanSourceInfo {
        rustc_span_cx: rustc_span.data().ctxt,
        rustc_start_offset: rustc_src.start_pos.0 as usize,
    };

    cx.storage.add_span_src_info(api_src, api_info);
    cx.storage.add_span_src(rustc_src.name.clone(), api_src);

    (api_src, api_info)
}

pub fn to_rustc_span<'ast, 'tcx: 'ast>(cx: &'ast RustcContext<'ast, 'tcx>, api_span: &Span<'ast>) -> rustc_span::Span {
    let src_info = cx
        .storage
        .span_src_info(api_span.source())
        .expect("all driver created `SpanSources` have a matching info");

    #[expect(clippy::cast_possible_truncation, reason = "`u32` is set by rustc and will be fine")]
    let lo = BytePos((api_span.start() + src_info.rustc_start_offset) as u32);
    #[expect(clippy::cast_possible_truncation, reason = "`u32` is set by rustc and will be fine")]
    let hi = BytePos((api_span.end() + src_info.rustc_start_offset) as u32);
    rustc_span::Span::new(lo, hi, src_info.rustc_span_cx, None)
}
#[derive(Debug, Clone, Copy)]
pub struct SpanSourceInfo {
    rustc_span_cx: rustc_span::hygiene::SyntaxContext,
    rustc_start_offset: usize,
}
