use marker_api::ast::pat::{
    CommonPatData, IdentPat, OrPat, PatKind, RefPat, RestPat, SlicePat, StructFieldPat, StructPat, TuplePat,
    UnstablePat, WildcardPat,
};
use rustc_hir as hir;

use super::MarkerConversionContext;

impl<'ast, 'tcx> MarkerConversionContext<'ast, 'tcx> {
    #[must_use]
    pub fn to_pat(&self, pat: &hir::Pat<'tcx>) -> PatKind<'ast> {
        // Here we don't need to take special care for caching, as marker patterns
        // don't have IDs and can't be requested individually. Instead patterns are
        // stored as part of their parent expressions or items. Not needing to deal
        // with caching makes this implementation simpler.
        let data = CommonPatData::new(self.to_span_id(pat.span));

        match &pat.kind {
            hir::PatKind::Wild => PatKind::Wildcard(self.alloc(|| WildcardPat::new(data))),
            hir::PatKind::Binding(hir::BindingAnnotation(by_ref, mutab), id, ident, pat) => {
                PatKind::Ident(self.alloc(|| {
                    IdentPat::new(
                        data,
                        self.to_symbol_id(ident.name),
                        self.to_var_id(*id),
                        matches!(mutab, rustc_ast::Mutability::Mut),
                        matches!(by_ref, hir::ByRef::Yes),
                        pat.map(|rustc_pat| self.to_pat(rustc_pat)),
                    )
                }))
            },
            hir::PatKind::Struct(qpath, fields, has_rest) => {
                let api_fields = self.alloc_slice_iter(fields.iter().map(|field| {
                    StructFieldPat::new(
                        self.to_span_id(field.span),
                        self.to_symbol_id(field.ident.name),
                        self.to_pat(field.pat),
                    )
                }));
                PatKind::Struct(
                    self.alloc(|| StructPat::new(data, self.to_path_from_qpath(qpath), api_fields, *has_rest)),
                )
            },
            hir::PatKind::TupleStruct(qpath, pats, dotdot) => {
                let ddpos = dotdot.as_opt_usize();
                let offset_pos = ddpos.unwrap_or(usize::MAX);
                let api_fields = self.alloc_slice_iter(pats.iter().enumerate().map(|(mut index, pat)| {
                    if index >= offset_pos {
                        index += offset_pos;
                    }
                    StructFieldPat::new(
                        self.to_span_id(pat.span),
                        self.to_symbol_id_for_num(u32::try_from(index).expect("a index over 2^32 is unexpected")),
                        self.to_pat(pat),
                    )
                }));
                PatKind::Struct(
                    self.alloc(|| StructPat::new(data, self.to_path_from_qpath(qpath), api_fields, ddpos.is_some())),
                )
            },
            hir::PatKind::Or(pats) => PatKind::Or(
                self.alloc(|| OrPat::new(data, self.alloc_slice_iter(pats.iter().map(|rpat| self.to_pat(rpat))))),
            ),
            hir::PatKind::Tuple(pats, dotdot) => {
                let pats = if let Some(rest_pos) = dotdot.as_opt_usize() {
                    let (start, end) = pats.split_at(rest_pos);
                    self.chain_pats(start, self.new_rest_pat(), end)
                } else {
                    self.alloc_slice_iter(pats.iter().map(|pat| self.to_pat(pat)))
                };
                PatKind::Tuple(self.alloc(|| TuplePat::new(data, pats)))
            },
            hir::PatKind::Box(_) => PatKind::Unstable(self.alloc(|| UnstablePat::new(data))),
            hir::PatKind::Ref(pat, muta) => {
                PatKind::Ref(self.alloc(|| RefPat::new(data, self.to_pat(pat), matches!(muta, hir::Mutability::Mut))))
            },
            hir::PatKind::Slice(start, wild, end) => {
                let elements = if let Some(wild) = wild {
                    self.chain_pats(start, self.to_pat(wild), end)
                } else {
                    assert!(end.is_empty());
                    self.alloc_slice_iter(start.iter().map(|pat| self.to_pat(pat)))
                };
                PatKind::Slice(self.alloc(|| SlicePat::new(data, elements)))
            },
            // These haven't been implemented yet, as they require expressions.
            // The pattern creation is tracked in #50
            hir::PatKind::Path(_) => todo!("{pat:#?}"),
            hir::PatKind::Lit(_) => todo!("{pat:#?}"),
            hir::PatKind::Range(_, _, _) => todo!("{pat:#?}"),
        }
    }

    fn chain_pats(
        &self,
        start: &[hir::Pat<'tcx>],
        ast_wild: PatKind<'ast>,
        end: &[hir::Pat<'tcx>],
    ) -> &'ast [PatKind<'ast>] {
        let start = start.iter().map(|pat| self.to_pat(pat));
        let middle = std::iter::once(ast_wild);
        let end = end.iter().map(|pat| self.to_pat(pat));
        let api_pats: Vec<_> = start.chain(middle).chain(end).collect();
        self.alloc_slice_iter(api_pats.into_iter())
    }

    fn new_rest_pat(&self) -> PatKind<'ast> {
        // This is a dummy span, it's dirty, but at least works for the mean time :)
        let data = CommonPatData::new(self.to_span_id(rustc_span::DUMMY_SP));
        PatKind::Rest(self.alloc(|| RestPat::new(data)))
    }
}
