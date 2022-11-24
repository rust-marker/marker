use linter_api::ast::{AstPath, AstPathSegment};
use rustc_hir as hir;

use crate::{context::RustcContext, conversion::generic::to_api_generic_args_opt};

use super::to_symbol_id;

pub fn to_api_path<'ast, 'tcx>(cx: &'ast RustcContext<'ast, 'tcx>, path: &hir::Path<'tcx>) -> AstPath<'ast> {
    AstPath::new(
        cx.storage
            .alloc_slice_iter(path.segments.iter().map(|seg| conv_segment(cx, seg))),
    )
}

fn conv_segment<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    segment: &hir::PathSegment<'tcx>,
) -> AstPathSegment<'ast> {
    AstPathSegment::new(
        to_symbol_id(segment.ident.name),
        to_api_generic_args_opt(cx, segment.args),
    )
}
