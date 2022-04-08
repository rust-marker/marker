#![allow(unused_variables)]

use std::fmt::Debug;

use linter_api::ast::{Attribute, CrateId, Lifetime, Span, Symbol};

use super::{rustc::RustcContext, ToApi};

impl<'ast, 'tcx> ToApi<'ast, 'tcx, CrateId> for rustc_hir::def_id::CrateNum {
    fn to_api(&self, _cx: &'ast RustcContext<'ast, 'tcx>) -> CrateId {
        CrateId::new(self.as_u32())
    }
}

#[derive(Debug)]
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

impl<'ast, 'tcx> ToApi<'ast, 'tcx, &'ast dyn Span<'ast>> for rustc_span::Span {
    fn to_api(&self, cx: &'ast RustcContext<'ast, 'tcx>) -> &'ast dyn Span<'ast> {
        cx.alloc_with(|| RustcSpan::new(*self, cx))
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

pub fn lifetime_from_hir<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    r_lt: rustc_hir::Lifetime,
) -> &'ast dyn Lifetime<'ast> {
    cx.new_lifetime()
}

impl<'ast, 'tcx> ToApi<'ast, 'tcx, Symbol> for rustc_span::Symbol {
    fn to_api(&self, _cx: &'ast RustcContext<'ast, 'tcx>) -> Symbol {
        Symbol::new(self.as_u32())
    }
}
