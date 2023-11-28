use marker_api::{diagnostic::DiagnosticBuilder, prelude::*, sem::UserDefinedTraitRef};

pub fn check_expr<'ast>(cx: &'ast MarkerContext<'ast>, expr: ExprKind<'ast>) {
    test_ty_impls_trait(cx, expr);
}

fn test_ty_impls_trait<'ast>(cx: &'ast MarkerContext<'ast>, input_expr: ExprKind<'ast>) {
    let ast::ExprKind::Path(expr) = input_expr else {
        return;
    };
    if !expr
        .path()
        .segments()
        .last()
        .map_or(false, |seg| seg.ident().name().starts_with("check_traits"))
    {
        return;
    }

    cx.emit_lint(super::TEST_TY_IMPLS_TRAIT, expr, "checking trait impls:")
        .decorate(|diag| {
            let ty: sem::TyKind<'_> = expr.ty();
            test_implements_trait(diag, ty, "std::marker::Sized", "");
            test_implements_trait(diag, ty, "std::marker::Send", "");
            test_implements_trait(diag, ty, "std::clone::Clone", "");
            test_implements_trait(diag, ty, "std::default::Default", "");
            test_implements_trait(diag, ty, "std::cmp::Ord", "");
            test_implements_trait(diag, ty, "unknown::Trait", "Should always be false");
            test_implements_trait(diag, ty, "crate::SimpleTestTrait", "");
            test_implements_trait(diag, ty, "crate::GenericTestTrait", "Path without generics");
            test_implements_trait(diag, ty, "crate::AssocTyTestTrait", "Path without type");
        });
}

fn test_implements_trait(
    diag: &mut DiagnosticBuilder<'_>,
    ty: sem::TyKind<'_>,
    path: impl Into<String>,
    comment: &str,
) {
    let path = path.into();
    diag.note(format!(
        "Implements: `{path}`: {} ({comment})",
        ty.implements_trait(&UserDefinedTraitRef::new(path.clone()))
    ));
}
