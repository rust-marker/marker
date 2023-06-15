#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]

use marker_api::{
    ast::{
        item::{
            ConstItem, EnumItem, EnumVariant, ExternCrateItem, Field, FnItem, ItemData, ItemKind, ModItem, StaticItem,
            StructItem, UseItem,
        },
        pat::PatKind,
        stmt::StmtKind,
        ty::SemTyKind,
        Span,
    },
    context::AstContext,
    diagnostic::{Applicability, EmissionNode},
    lint::Lint,
    LintPass,
};

marker_api::interface::export_lint_pass!(TestLintPass);

marker_api::lint::declare_lint!(TEST_LINT, Warn, "test lint warning");

// this ideally should use only specific `check_*` functions
// to test them, think `check_struct` instead of `check_item`
marker_api::lint::declare_lint!(FOO_ITEMS, Warn, "non-descriptive item names");

fn emit_foo_lint<'ast, S: Into<String>>(
    cx: &'ast AstContext<'ast>,
    node: impl Into<EmissionNode>,
    description: S,
    span: &Span<'ast>,
) {
    let msg = description.into() + " named `foo`, consider using a more meaningful name";
    cx.emit_lint(FOO_ITEMS, node, msg, span, |_| {});
}

#[derive(Default)]
struct TestLintPass {}

impl LintPass for TestLintPass {
    fn registered_lints(&self) -> Box<[&'static Lint]> {
        Box::new([TEST_LINT])
    }

    fn check_static_item<'ast>(&mut self, cx: &'ast AstContext<'ast>, item: &'ast StaticItem<'ast>) {
        if let Some(name) = item.ident() {
            let name = name.name();
            if name.starts_with("PRINT_TYPE") {
                cx.emit_lint(TEST_LINT, item.id(), "Printing type for", item.ty().span(), |_| {});
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
            } else if name == "FOO" {
                emit_foo_lint(cx, item.id(), "a static item", item.span());
            }
        }
    }

    fn check_const_item<'ast>(&mut self, cx: &'ast AstContext<'ast>, item: &'ast ConstItem<'ast>) {
        if matches!(item.ident(), Some(name) if name == "FOO") {
            emit_foo_lint(cx, item.id(), "a constant item", item.span());
        }
    }

    fn check_extern_crate<'ast>(&mut self, cx: &'ast AstContext<'ast>, item: &'ast ExternCrateItem<'ast>) {
        if matches!(item.ident(), Some(name) if name == "foo") {
            emit_foo_lint(cx, item.id(), "an `extern` crate", item.span());
        }
    }

    fn check_use_decl<'ast>(&mut self, cx: &'ast AstContext<'ast>, item: &'ast UseItem<'ast>) {
        if item.is_glob() {
            return;
        }
        if matches!(item.ident(), Some(name) if name == "foo") {
            emit_foo_lint(cx, item.id(), "a `use` binding", item.span());
        }
    }

    fn check_field<'ast>(&mut self, cx: &'ast AstContext<'ast>, field: &'ast Field<'ast>) {
        if field.ident() == "foo" {
            emit_foo_lint(cx, field.id(), "a field", field.span());
        }
    }

    fn check_variant<'ast>(&mut self, cx: &'ast AstContext<'ast>, variant: &'ast EnumVariant<'ast>) {
        if variant.ident() == "Foo" {
            emit_foo_lint(cx, variant.id(), "an enum variant", variant.span());
        }
    }

    fn check_mod<'ast>(&mut self, cx: &'ast AstContext<'ast>, item: &'ast ModItem<'ast>) {
        if matches!(item.ident(), Some(name) if name == "foo") {
            emit_foo_lint(cx, item.id(), "a module", item.span());
        }
    }

    fn check_enum<'ast>(&mut self, cx: &'ast AstContext<'ast>, item: &'ast EnumItem<'ast>) {
        if matches!(item.ident(), Some(name) if name == "Foo") {
            emit_foo_lint(cx, item.id(), "an enum", item.span());
        }
    }

    fn check_struct<'ast>(&mut self, cx: &'ast AstContext<'ast>, item: &'ast StructItem<'ast>) {
        if matches!(item.ident(), Some(name) if name == "Foo") {
            emit_foo_lint(cx, item.id(), "a struct", item.span());
        }
    }

    fn check_fn<'ast>(&mut self, cx: &'ast AstContext<'ast>, item: &'ast FnItem<'ast>) {
        if matches!(item.ident(), Some(name) if name == "foo") {
            emit_foo_lint(cx, item.id(), "a function", item.span());
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
                    let SemTyKind::Adt(adt) = expr.ty().kind() else {
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
                });
            }
        }
    }

    fn check_item<'ast>(&mut self, cx: &'ast AstContext<'ast>, item: ItemKind<'ast>) {
        fn try_resolve_path(cx: &AstContext<'_>, path: &str) {
            let ids = cx.resolve_ty_ids(path);
            eprintln!("Resolving {path:?} yielded {ids:#?}");
        }

        if let ItemKind::Fn(item) = item {
            if let Some(ident) = item.ident() {
                if ident.name() == "test_ty_id_resolution" {
                    eprintln!("# Invalid paths");
                    try_resolve_path(cx, "");
                    try_resolve_path(cx, "something");
                    try_resolve_path(cx, "bool");
                    try_resolve_path(cx, "u32");
                    try_resolve_path(cx, "crate::super");
                    try_resolve_path(cx, "crate::self::super");

                    eprintln!("");
                    eprintln!("# Unresolvable");
                    try_resolve_path(cx, "something::weird");
                    try_resolve_path(cx, "something::weird::very::very::very::very::very::long");

                    eprintln!("");
                    eprintln!("# Not a type");
                    try_resolve_path(cx, "std::env");
                    try_resolve_path(cx, "std::i32");
                    try_resolve_path(cx, "std::primitive::i32");
                    try_resolve_path(cx, "std::option::Option::None");

                    eprintln!("");
                    eprintln!("# Valid");
                    try_resolve_path(cx, "std::option::Option");
                    try_resolve_path(cx, "std::vec::Vec");
                    try_resolve_path(cx, "std::string::String");

                    eprintln!("");
                    eprintln!("=====================================================================");
                    eprintln!("");
                }
            }
        }
    }
}
