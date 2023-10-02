#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]

mod utils;

use marker_api::{
    ast::{
        item::{EnumVariant, Field, StaticItem},
        stmt::LetStmt,
        AstPathTarget,
    },
    diagnostic::Applicability,
    prelude::*,
    sem::ty::SemTyKind,
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
    /// A lint used for marker's uitests.
    ///
    /// It's used to print spans and is allowed to emit code in macros
    PRINT_SPAN_LINT,
    Warn,
    marker_api::lint::MacroReport::All,
}

marker_api::declare_lint! {
    /// # What it does
    /// A lint used for markers uitests.
    ///
    /// It warns about about item names starting with `FindMe`, `find_me` or `FIND_ME`.
    ITEM_WITH_TEST_NAME,
    Warn,
}

marker_api::declare_lint! {
    /// # What it does
    /// A lint used for markers uitests.
    ///
    /// It prints out every expression, if this lint is set to warn at the
    /// expression node.
    PRINT_EVERY_EXPR,
    Allow,
}

marker_api::declare_lint! {
    /// # What it does
    /// A lint used for marker's uitests.
    ///
    /// A lint to test [`marker_api::AstMap`].
    TEST_AST_MAP,
    Warn,
}

fn emit_item_with_test_name_lint<'ast>(
    cx: &'ast MarkerContext<'ast>,
    node: impl EmissionNode<'ast>,
    desc: impl std::fmt::Display,
) {
    let msg = format!("found {desc} with a test name");
    cx.emit_lint(ITEM_WITH_TEST_NAME, node, msg);
}

impl LintPass for TestLintPass {
    fn info(&self) -> LintPassInfo {
        LintPassInfoBuilder::new(Box::new([
            TEST_LINT,
            ITEM_WITH_TEST_NAME,
            PRINT_EVERY_EXPR,
            utils::TEST_CONTAINS_RETURN,
        ]))
        .build()
    }

    fn check_item<'ast>(&mut self, cx: &'ast MarkerContext<'ast>, item: ItemKind<'ast>) {
        utils::check_item(cx, item);

        if let ItemKind::Fn(item) = item {
            if let Some(ident) = item.ident() {
                if ident.name() == "test_ty_id_resolution_trigger" {
                    test_ty_id_resolution(cx);
                } else if ident.name() == "uilints_please_ice_on_this" {
                    panic!("free ice cream for everyone!!!");
                }
            }
        }

        if let ItemKind::Static(item) = item {
            check_static_item(cx, item);
        }

        if matches!(
            item.ident().map(marker_api::span::Ident::name),
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
                emit_item_with_test_name_lint(cx, item, format!("a `{msg}` item"));
            }
        }

        if matches!(
            item.ident().map(marker_api::span::Ident::name),
            Some(name) if name.starts_with("PrintMe") || name.starts_with("PRINT_ME") || name.starts_with("print_me")
        ) {
            cx.emit_lint(TEST_LINT, item, "printing item").decorate(|diag| {
                diag.span(item.ident().unwrap().span());
                diag.note(format!("{item:#?}"));
            });
        }

        if let ItemKind::Fn(func) = item {
            if matches!(
                item.ident().map(marker_api::span::Ident::name),
                Some(name) if name.starts_with("print_with_body")
            ) {
                cx.emit_lint(TEST_LINT, item, "printing item with body")
                    .decorate(|diag| {
                        diag.span(item.ident().unwrap().span());
                        diag.note(format!("Item: {item:#?}"));
                        diag.note(format!("Body: {:#?}", cx.ast().body(func.body_id().unwrap())));
                    });
            }
        }
    }

    fn check_field<'ast>(&mut self, cx: &'ast MarkerContext<'ast>, field: &'ast Field<'ast>) {
        if field.ident().starts_with("find_me") {
            emit_item_with_test_name_lint(cx, field, "a field");
        }
    }

    fn check_variant<'ast>(&mut self, cx: &'ast MarkerContext<'ast>, variant: &'ast EnumVariant<'ast>) {
        if variant.ident().starts_with("FindMe") {
            emit_item_with_test_name_lint(cx, variant, "an enum variant");
        }
    }

    fn check_stmt<'ast>(&mut self, cx: &'ast MarkerContext<'ast>, stmt: StmtKind<'ast>) {
        // I didn't realize that `let_chains` are still unstable. This makes the
        // code significantly less readable -.-
        if let StmtKind::Let(lets) = stmt {
            let PatKind::Ident(ident) = lets.pat() else { return };
            let Some(expr) = lets.init() else { return };
            if ident.name().starts_with("_print") {
                cx.emit_lint(TEST_LINT, stmt, "print test").decorate(|diag| {
                    diag.note(format!("{expr:#?}"));
                });
            } else if ident.name().starts_with("_span") {
                cx.emit_lint(PRINT_SPAN_LINT, stmt, "print span").decorate(|diag| {
                    let span = expr.span();
                    diag.note(format!("Debug: {span:#?}"));
                    diag.note(format!("Snippet: {}", span.snippet_or("..")));
                    diag.note(format!("Source: {:#?}", span.source()));
                });
            } else if ident.name().starts_with("_ty") {
                cx.emit_lint(TEST_LINT, stmt, "print type test").decorate(|diag| {
                    diag.note(format!("{:#?}", expr.ty()));
                });
            } else if ident.name().starts_with("_check_path") {
                cx.emit_lint(TEST_LINT, stmt, "check type resolution").decorate(|diag| {
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
            } else if ident.name().starts_with("_check_ast_map") {
                check_ast_map(cx, lets);
            }
        }
    }

    fn check_expr<'ast>(&mut self, cx: &'ast MarkerContext<'ast>, expr: ExprKind<'ast>) {
        cx.emit_lint(PRINT_EVERY_EXPR, expr, "expr").decorate(|diag| {
            diag.note(&format!("SpanSource: {:#?}", expr.span().source()));
            diag.note(&format!("Snippet: {:#?}", expr.span().snippet_or("<..>")));
        });
    }
}

fn check_ast_map<'ast>(cx: &'ast MarkerContext<'ast>, lets: &'ast LetStmt<'ast>) {
    let Some(expr) = lets.init() else { return };

    match expr {
        ExprKind::Ctor(ctor) => {
            let path = ctor.path();
            match path.resolve() {
                AstPathTarget::Variant(var_id) => cx
                    .emit_lint(TEST_AST_MAP, expr, "testing `AstMap::variant`")
                    .decorate(|diag| {
                        diag.note(format!("`AstMap::variant()` --> {:#?}", cx.ast().variant(var_id)))
                            .done();
                    })
                    .done(),
                AstPathTarget::Item(item_id) => cx
                    .emit_lint(TEST_AST_MAP, expr, "testing `AstMap::item`")
                    .decorate(|diag| {
                        diag.note(format!("`AstMap::item()` --> {:#?}", cx.ast().item(item_id)))
                            .done();
                    })
                    .done(),
                _ => unreachable!(),
            }
        },
        _ => {
            unreachable!()
        },
    }
}

fn check_static_item<'ast>(cx: &'ast MarkerContext<'ast>, item: &'ast StaticItem<'ast>) {
    if let Some(name) = item.ident() {
        let name = name.name();
        if name.starts_with("PRINT_TYPE") {
            cx.emit_lint(TEST_LINT, item, "printing type for").decorate(|diag| {
                diag.span(item.ty().span());
            });
            eprintln!("{:#?}\n\n", item.ty());
        } else if name.starts_with("FIND_ITEM") {
            cx.emit_lint(TEST_LINT, item, "hey there is a static item here")
                .decorate(|diag| {
                    diag.note("a note");
                    diag.help("a help");
                    diag.span_note("a spanned note", item.span());
                    diag.span_help("a spanned help", item.span());
                    diag.span_suggestion("try", item.span(), "duck", Applicability::Unspecified);
                });
        }
    }
}

fn test_ty_id_resolution<'ast>(cx: &'ast MarkerContext<'ast>) {
    fn try_resolve_path(cx: &MarkerContext<'_>, path: &str) {
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
