#![allow(unused_variables)]

use std::fmt::Debug;

use linter_api::ast::{Attribute, Lifetime};

use super::rustc::RustcContext;

pub struct RustcSpan<'ast, 'tcx> {
    span: rustc_span::Span,
    cx: &'ast RustcContext<'ast, 'tcx>,
}

impl<'ast, 'tcx> RustcSpan<'ast, 'tcx> {
    #[must_use]
    pub fn new(span: rustc_span::Span, cx: &'ast RustcContext<'ast, 'tcx>) -> Self {
        Self { span, cx }
    }
}

impl Debug for RustcSpan<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustcSpan").field("span", &self.span).finish()
    }
}

impl<'ast, 'tcx> linter_api::ast::Span<'ast> for RustcSpan<'ast, 'tcx> {
    fn is_from_expansion(&self) -> bool {
        self.span.from_expansion()
    }

    fn in_derive_expansion(&self) -> bool {
        self.span.in_derive_expansion()
    }

    fn contains(&self, other: &dyn linter_api::ast::Span<'ast>) -> bool {
        todo!()
    }

    fn overlaps(&self, other: &dyn linter_api::ast::Span<'ast>) -> bool {
        todo!()
    }

    fn edition(&self) -> linter_api::ast::Edition {
        todo!()
    }

    fn to(&'ast self, end: &dyn linter_api::ast::Span<'ast>) -> &dyn linter_api::ast::Span<'ast> {
        todo!()
    }

    fn between(&'ast self, end: &dyn linter_api::ast::Span<'ast>) -> &dyn linter_api::ast::Span<'ast> {
        todo!()
    }

    fn until(&'ast self, end: &dyn linter_api::ast::Span<'ast>) -> &dyn linter_api::ast::Span<'ast> {
        todo!()
    }

    fn snippet(&self) -> Option<String> {
        self.cx.tcx.sess.source_map().span_to_snippet(self.span).ok()
    }

    fn get_source_file(&self) -> Option<(String, u32, u32)> {
        todo!()
    }
}

#[derive(Debug)]
pub struct RustcAttribute {}

impl<'ast> Attribute<'ast> for RustcAttribute {}

#[derive(Debug)]
pub struct RustcLifetime {}

impl<'ast> Lifetime<'ast> for RustcLifetime {}

pub fn lifetime_from_region<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    _reg: rustc_middle::ty::Region<'tcx>,
) -> &'ast dyn Lifetime<'ast> {
    cx.new_lifetime()
}
