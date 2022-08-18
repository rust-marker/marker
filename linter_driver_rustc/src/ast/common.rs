#![allow(unused_variables)]

use std::{fmt::Debug, mem::transmute};

use linter_api::ast::{
    Attribute, BodyId, CrateId, ItemPath, Lifetime, PathResolution, PathSegment, Span, SpanId, SpanSource, Symbol,
};

use super::{rustc::RustcContext, ToApi};
use rustc_lint::LintContext;

impl<'ast, 'tcx> ToApi<'ast, 'tcx, CrateId> for rustc_hir::def_id::CrateNum {
    fn to_api(&self, _cx: &'ast RustcContext<'ast, 'tcx>) -> CrateId {
        CrateId::new(self.as_u32())
    }
}

impl<'ast, 'tcx> ToApi<'ast, 'tcx, BodyId> for rustc_hir::BodyId {
    fn to_api(&self, _cx: &'ast RustcContext<'ast, 'tcx>) -> BodyId {
        let (x1, x2) = self.hir_id.index();
        BodyId::new(x1, x2)
    }
}

#[expect(clippy::missing_panics_doc)]
pub fn api_span_from_rustc_span<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    span: rustc_span::Span,
) -> &'ast Span<'ast> {
    // FIXME: Calling this code "hacky" would be an understatement... Cleaning
    // this code up is sadly not that easy and requires a restructuring of
    // this driver. This is high on the TODO list. Actually, the next thing after
    // a rustc sync but sadly after reworking the lint-crate -> driver
    // communication. Therefore, I'm adding/leaving this hack as my next TODO to
    // make progress on the other front.

    let rustc_data = span.data();
    let span_file = cx.rustc_cx.sess().source_map().lookup_source_file(span.lo());
    if let rustc_span::FileName::Real(path) = &span_file.name {
        let thing = path
            .clone()
            .into_local_path()
            .as_ref()
            .map(std::clone::Clone::clone)
            .unwrap();
        let thing = cx.alloc_with(move || thing);
        let source = SpanSource::File(thing);
        let start = span.lo() - span_file.start_pos;
        let end = span.hi() - span_file.start_pos;
        return cx.alloc_with(|| Span::new(cx.ast_cx(), source, start.0 as usize, end.0 as usize));
    }
    unimplemented!()
}

#[must_use]
pub fn rustc_span_from_span_id(span_id: SpanId) -> rustc_span::Span {
    // # Safety
    //
    // [`SpanId`]s are only created by [`span_id_from_rustc_span`] which uses
    // the same transmute. The size of these structs are validated in an extra
    // check.
    //
    // FIXME: In theory this is still unsound since [`rustc_span::Span`] doesn't
    // have `[repr(C)]`. For a stable release this should be fixed. Until then this
    // is as simple and safe as it gets.
    unsafe { transmute::<SpanId, rustc_span::Span>(span_id) }
}

#[must_use]
pub fn span_id_from_rustc_span(rustc_span: rustc_span::Span) -> SpanId {
    // # Safety
    //
    // The here created [`SpanId`]s are transformed back by [`rustc_span_from_span_id`]
    // which uses the same transmute. The size of these structs are validated in an extra
    // test.
    unsafe { transmute::<rustc_span::Span, SpanId>(rustc_span) }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    /// These tests validate that the size of transmutation sources and targets
    /// doesn't change unexpected. If a size is changed all transmutational usages
    /// will need to be checked.
    #[test]
    pub fn check_transmute_target_sizes() {
        assert_eq!(size_of::<rustc_span::Span>(), 8, "Test `rustc_span::Span`");
        assert_eq!(
            size_of::<linter_api::ast::SpanId>(),
            8,
            "Test `linter_api::ast::SpanId`"
        );
    }
}

pub fn path_from_rustc<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    inner: &'tcx rustc_hir::Path<'tcx>,
) -> ItemPath<'ast> {
    let target = inner.res.to_api(cx);
    let segments = cx.alloc_slice_from_iter(inner.segments.iter().map(|seg| seg.to_api(cx)));
    ItemPath::new(segments, target)
}

impl<'ast, 'tcx> ToApi<'ast, 'tcx, PathSegment> for rustc_hir::PathSegment<'tcx> {
    fn to_api(&self, cx: &'ast RustcContext<'ast, 'tcx>) -> PathSegment {
        let name = self.ident.name.to_api(cx);
        let target = self.res.to_api(cx);
        PathSegment::new(name, target)
    }
}

impl<'ast, 'tcx> ToApi<'ast, 'tcx, PathResolution> for Option<rustc_hir::def::Res> {
    fn to_api(&self, cx: &'ast RustcContext<'ast, 'tcx>) -> PathResolution {
        match self {
            Option::Some(res) => res.to_api(cx),
            Option::None => PathResolution::Unresolved,
        }
    }
}

impl<'ast, 'tcx> ToApi<'ast, 'tcx, PathResolution> for rustc_hir::def::Res {
    fn to_api(&self, cx: &'ast RustcContext<'ast, 'tcx>) -> PathResolution {
        match self {
            rustc_hir::def::Res::Def(_def_kind, def_id) => PathResolution::Item(def_id.to_api(cx)),
            rustc_hir::def::Res::ToolMod => PathResolution::ToolItem,
            // rustc_hir::def::Res::PrimTy(PrimTy),
            // rustc_hir::def::Res::SelfTy {..},
            // rustc_hir::def::Res::SelfCtor(DefId),
            // rustc_hir::def::Res::Local(Id),
            // rustc_hir::def::Res::NonMacroAttr(NonMacroAttrKind),
            rustc_hir::def::Res::Err => PathResolution::Unresolved,
            _ => todo!(),
        }
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
