#![allow(unused_variables)]

use std::fmt::Debug;

use linter_api::ast::{Attribute, BodyId, CrateId, Lifetime, Path, PathResolution, PathSegment, Span, Symbol};

use super::{rustc::RustcContext, ToApi};

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

pub fn path_from_rustc<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    inner: &'tcx rustc_hir::Path<'tcx>,
) -> Path<'ast> {
    let target = inner.res.to_api(cx);
    let segments = cx.alloc_slice_from_iter(inner.segments.iter().map(|seg| seg.to_api(cx)));
    Path::new(segments, target)
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
