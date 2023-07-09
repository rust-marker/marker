#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]

use marker_api::{
    ast::{
        item::{EnumVariant, Field, ItemData, ItemKind, StaticItem},
        pat::PatKind,
        stmt::StmtKind,
        ty::SemTyKind,
        Span,
    },
    context::AstContext,
    diagnostic::{Applicability, EmissionNode},
    LintPass, LintPassInfo, LintPassInfoBuilder,
};

#[derive(Default)]
struct TestLintPass {}

marker_api::export_lint_pass!(TestLintPass);

marker_api::declare_lint! {
    /// # What it does
    /// A lint used for marker's uitests.
    ///
    /// It's just the default lint used for most emissions here :)
    TEST_LINT,
    Warn,
}

marker_api::declare_lint! {
    /// # What it does
    /// A lint used for markers uitests.
    ///
    /// It warns about about item names starting with `FindMe`, `find_me` or `FIND_ME`.
    ITEM_WITH_TEST_NAME,
    Warn,
}

fn emit_item_with_test_name_lint<'ast>(
    cx: &'ast AstContext<'ast>,
    node: impl Into<EmissionNode>,
    desc: impl std::fmt::Display,
    span: &Span<'ast>,
) {
    let msg = format!("found {desc} with a test name");
    cx.emit_lint(ITEM_WITH_TEST_NAME, node, msg, span, |_| {});
}

impl LintPass for TestLintPass {
    fn info(&self) -> LintPassInfo {
        LintPassInfoBuilder::new(Box::new([TEST_LINT, ITEM_WITH_TEST_NAME])).build()
    }

    fn check_item<'ast>(&mut self, cx: &'ast AstContext<'ast>, item: ItemKind<'ast>) {
        if let ItemKind::Fn(item) = item {
            if let Some(ident) = item.ident() {
                if ident.name() == "test_ty_id_resolution_trigger" {
                    test_ty_id_resolution(cx);
                }
            }
        }

        if let ItemKind::Static(item) = item {
            check_static_item(cx, item);
        }

        if matches!(
            item.ident().map(marker_api::ast::Ident::name),
            Some(name) if name.starts_with("FindMe") || name.starts_with("FIND_ME") || name.starts_with("find_me")
        ) {
            let msg = match item {
                ItemKind::Mod(_) => Some("module"),
                ItemKind::Use(_) => Some("use"),
                ItemKind::Static(_) => Some("static"),
                ItemKind::Const(_) => Some("const"),
                ItemKind::Fn(_) => Some("fn"),
                ItemKind::Struct(_) => Some("struct"),
                ItemKind::Enum(_) => Some("enum"),
                ItemKind::Union(_) => Some("union"),
                ItemKind::Trait(_) => Some("trait"),
                _ => None,
            };

            if let Some(msg) = msg {
                emit_item_with_test_name_lint(cx, item.id(), format!("a `{msg}` item"), item.span());
            }
        }
    }

    fn check_field<'ast>(&mut self, cx: &'ast AstContext<'ast>, field: &'ast Field<'ast>) {
        if field.ident().starts_with("find_me") {
            emit_item_with_test_name_lint(cx, field.id(), "a field", field.span());
        }
    }

    fn check_variant<'ast>(&mut self, cx: &'ast AstContext<'ast>, variant: &'ast EnumVariant<'ast>) {
        if variant.ident().starts_with("FindMe") {
            emit_item_with_test_name_lint(cx, variant.id(), "an enum variant", variant.span());
        }
    }

    fn check_stmt<'ast>(&mut self, cx: &'ast AstContext<'ast>, stmt: StmtKind<'ast>) {
        // I didn't realize that `let_chains` are still unstable. This makes the
        // code significantly less readable -.-
        if let StmtKind::Let(lets) = stmt {
            let PatKind::Ident(ident) = lets.pat() else { return };
            let Some(expr) = lets.init() else { return };
            if ident.name().starts_with("_print") {
                cx.emit_lint(TEST_LINT, stmt.id(), "print test", stmt.span(), |diag| {
                    diag.note(format!("{expr:#?}"));
                });
            } else if ident.name().starts_with("_ty") {
                cx.emit_lint(TEST_LINT, stmt.id(), "print type test", stmt.span(), |diag| {
                    diag.note(format!("{:#?}", expr.ty()));
                });
            } else if ident.name().starts_with("_check_path") {
                cx.emit_lint(TEST_LINT, stmt.id(), "check type resolution", stmt.span(), |diag| {
                    let SemTyKind::Adt(adt) = expr.ty() else {
                        unreachable!("how? Everything should be an ADT")
                    };
                    let path = "std::vec::Vec";
                    let ids = cx.resolve_ty_ids(path);
                    diag.note(format!("Is this a {:#?} -> {}", path, ids.contains(&adt.def_id())));

                    let path = "std::string::String";
                    let ids = cx.resolve_ty_ids(path);
                    diag.note(format!("Is this a {:#?} -> {}", path, ids.contains(&adt.def_id())));

                    let path = "std::option::Option";
                    let ids = cx.resolve_ty_ids(path);
                    diag.note(format!("Is this a {:#?} -> {}", path, ids.contains(&adt.def_id())));

                    let path = "crate::TestType";
                    let ids = cx.resolve_ty_ids(path);
                    diag.note(format!("Is this a {:#?} -> {}", path, ids.contains(&adt.def_id())));
                });
            }
        }
    }
}

fn check_static_item<'ast>(cx: &'ast AstContext<'ast>, item: &'ast StaticItem<'ast>) {
    if let Some(name) = item.ident() {
        let name = name.name();
        if name.starts_with("PRINT_TYPE") {
            cx.emit_lint(TEST_LINT, item.id(), "printing type for", item.ty().span(), |_| {});
            eprintln!("{:#?}\n\n", item.ty());
        } else if name.starts_with("FIND_ITEM") {
            cx.emit_lint(
                TEST_LINT,
                item.id(),
                "hey there is a static item here",
                item.span(),
                |diag| {
                    diag.note("a note");
                    diag.help("a help");
                    diag.span_note("a spanned note", item.span());
                    diag.span_help("a spanned help", item.span());
                    diag.span_suggestion("try", item.span(), "duck", Applicability::Unspecified);
                },
            );
        }
    }
}

fn test_ty_id_resolution<'ast>(cx: &'ast AstContext<'ast>) {
    fn try_resolve_path(cx: &AstContext<'_>, path: &str) {
        let ids = cx.resolve_ty_ids(path);
        eprintln!("Resolving {path:?} yielded {ids:#?}");
    }

    eprintln!("# Invalid paths");
    try_resolve_path(cx, "");
    try_resolve_path(cx, "something");
    try_resolve_path(cx, "bool");
    try_resolve_path(cx, "u32");
    try_resolve_path(cx, "crate::super");
    try_resolve_path(cx, "crate::self::super");

    eprintln!();
    eprintln!("# Unresolvable");
    try_resolve_path(cx, "something::weird");
    try_resolve_path(cx, "something::weird::very::very::very::very::very::long");

    eprintln!();
    eprintln!("# Not a type");
    try_resolve_path(cx, "std::env");
    try_resolve_path(cx, "std::i32");
    try_resolve_path(cx, "std::primitive::i32");
    try_resolve_path(cx, "std::option::Option::None");

    eprintln!();
    eprintln!("# Valid");
    try_resolve_path(cx, "std::option::Option");
    try_resolve_path(cx, "std::vec::Vec");
    try_resolve_path(cx, "std::string::String");

    eprintln!();
    eprintln!("# Valid local items");
    try_resolve_path(cx, "item_id_resolution::TestType");
    try_resolve_path(cx, "crate::TestType");
    eprintln!(
        "Check equal: {}",
        cx.resolve_ty_ids("item_id_resolution::TestType") == cx.resolve_ty_ids("crate::TestType")
    );

    eprintln!();
    eprintln!("=====================================================================");
    eprintln!();
}
