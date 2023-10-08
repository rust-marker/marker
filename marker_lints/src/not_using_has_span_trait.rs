use marker_api::prelude::*;

marker_api::declare_lint! {
    /// ### What it does
    /// Suggests using `impl HasSpan` for functions that need to take `Span` as
    /// a parameter to make them more ergonomic.
    NOT_USING_HAS_SPAN_TRAIT,
    // FIXME(#26): This allow by default until we have the item visibility info
    // in the AST. Once that is in place the lint should be made `warn` by default,
    // and it should ignore non `pub` functions.
    Allow,
}

pub(crate) fn check_item<'ast>(cx: &'ast MarkerContext<'ast>, item: ItemKind<'ast>) {
    let ItemKind::Fn(func) = item else { return };

    for param in func.params() {
        let ast::TyKind::Path(path) = param.ty().peel_refs() else {
            continue;
        };

        let ident = path
            .path()
            .segments()
            .last()
            .unwrap_or_else(|| panic!("BUG: path must have at least one segment, but got: {path:#?}"))
            .ident();

        if ident != "Span" {
            continue;
        }

        cx.emit_lint(
            NOT_USING_HAS_SPAN_TRAIT,
            func,
            "use impl HasSpan instead of Span for more flexibility",
        )
        .span(param.ty().span());
    }
}
