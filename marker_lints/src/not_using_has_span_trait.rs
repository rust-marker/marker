use marker_api::prelude::*;

marker_api::declare_lint! {
    /// ### What it does
    /// Suggests using `impl HasSpan` for functions that need to take `Span` as
    /// a parameter to make them more ergonomic.
    ///
    /// This function only triggers on public items, as it's targeted to towards
    /// the public interface of crates.
    NOT_USING_HAS_SPAN_TRAIT,
    Allow,
}

pub(crate) fn check_item<'ast>(cx: &'ast MarkerContext<'ast>, item: ItemKind<'ast>) {
    let ItemKind::Fn(func) = item else { return };
    if !func.visibility().is_pub() {
        return;
    }

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
