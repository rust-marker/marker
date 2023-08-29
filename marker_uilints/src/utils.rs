use marker_api::prelude::*;
use marker_utils::search::contains_return;

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
        let res = contains_return(cx, cx.body(fn_item.body_id().unwrap()).expr());

        cx.emit_lint(
            TEST_CONTAINS_RETURN,
            item.id(),
            format!("testing `contains_return` -> {res}"),
            ident.span(),
            |_| {},
        )
    }
}
