use marker_api::prelude::*;
use marker_utils::visitor::BoolTraversable;

marker_api::declare_lint! {
    /// # What it does
    /// Tests the [`marker_utils::search::contains_return`] function.
    TEST_CONTAINS_RETURN,
    Warn,
}

pub fn check_item<'ast>(cx: &'ast AstContext<'ast>, item: ItemKind<'ast>) {
    let ItemKind::Fn(fn_item) = item else { return };
    let Some(ident) = fn_item.ident() else { return };

    if ident.name().starts_with("test_contains_return") {
        let body = cx.body(fn_item.body_id().unwrap());
        let res = body.contains_return(cx);

        cx.emit_lint(
            TEST_CONTAINS_RETURN,
            item,
            format!("testing `contains_return` -> {res}"),
        )
        .decorate(|diag| {
            diag.set_main_span(ident.span());
        });
    }
}
