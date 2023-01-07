#![doc = include_str!("../README.md")]
#![warn(clippy::pedantic)]

use marker_api::{
    ast::{
        item::{
            ConstItem, EnumItem, EnumVariant, ExternCrateItem, Field, FnItem, ItemData, ModItem, StaticItem,
            StructItem, UseItem,
        },
        Span,
    },
    context::AstContext,
    lint::Lint,
    LintPass,
};

marker_api::interface::export_lint_pass!(TestLintPass);

marker_api::lint::declare_lint!(TEST_LINT, Warn, "test lint warning");

// this ideally should use only specific `check_*` functions
// to test them, think `check_struct` instead of `check_item`
marker_api::lint::declare_lint!(FOO_ITEMS, Warn, "non-descriptive item names");

fn emit_foo_lint<'ast, S: Into<String>>(cx: &'ast AstContext<'ast>, description: S, span: &'ast Span) {
    let msg = description.into() + " named `foo`, consider using a more meaningful name";
    cx.emit_lint(FOO_ITEMS, &msg, span);
}

#[derive(Default)]
struct TestLintPass {}

impl<'ast> LintPass<'ast> for TestLintPass {
    fn registered_lints(&self) -> Box<[&'static Lint]> {
        Box::new([TEST_LINT])
    }

    fn check_static_item(&mut self, cx: &'ast AstContext<'ast>, item: &'ast StaticItem<'ast>) {
        if let Some(name) = item.ident() {
            let name = name.name();
            if name.starts_with("PRINT_TYPE") {
                cx.emit_lint(TEST_LINT, "Printing type for", item.ty().span().unwrap());
                eprintln!("{:#?}\n\n", item.ty());
            } else if name.starts_with("FIND_ITEM") {
                cx.emit_lint(TEST_LINT, "hey there is a static item here", item.span());
            } else if name == "FOO" {
                emit_foo_lint(cx, "a static item", item.span());
            }
        }
    }

    fn check_const_item(&mut self, cx: &'ast AstContext<'ast>, item: &'ast ConstItem<'ast>) {
        if matches!(item.ident(), Some(name) if name == "FOO") {
            emit_foo_lint(cx, "a constant item", item.span());
        }
    }

    fn check_extern_crate(&mut self, cx: &'ast AstContext<'ast>, item: &'ast ExternCrateItem<'ast>) {
        if matches!(item.ident(), Some(name) if name == "foo") {
            emit_foo_lint(cx, "an `extern` crate", item.span());
        }
    }

    fn check_use_decl(&mut self, cx: &'ast AstContext<'ast>, item: &'ast UseItem<'ast>) {
        if item.is_glob() {
            return;
        }
        if matches!(item.ident(), Some(name) if name == "foo") {
            emit_foo_lint(cx, "a `use` binding", item.span());
        }
    }

    fn check_field(&mut self, cx: &'ast AstContext<'ast>, field: &'ast Field<'ast>) {
        if field.ident() == "foo" {
            emit_foo_lint(cx, "a field", field.span());
        }
    }

    fn check_variant(&mut self, cx: &'ast AstContext<'ast>, variant: &'ast EnumVariant<'ast>) {
        if variant.ident() == "Foo" {
            emit_foo_lint(cx, "an enum variant", variant.span());
        }
    }

    fn check_mod(&mut self, cx: &'ast AstContext<'ast>, item: &'ast ModItem<'ast>) {
        if matches!(item.ident(), Some(name) if name == "foo") {
            emit_foo_lint(cx, "a module", item.span());
        }
    }

    fn check_enum(&mut self, cx: &'ast AstContext<'ast>, item: &'ast EnumItem<'ast>) {
        if matches!(item.ident(), Some(name) if name == "Foo") {
            emit_foo_lint(cx, "an enum", item.span());
        }
    }

    fn check_struct(&mut self, cx: &'ast AstContext<'ast>, item: &'ast StructItem<'ast>) {
        if matches!(item.ident(), Some(name) if name == "Foo") {
            emit_foo_lint(cx, "a struct", item.span());
        }
    }

    fn check_fn(&mut self, cx: &'ast AstContext<'ast>, item: &'ast FnItem<'ast>) {
        if matches!(item.ident(), Some(name) if name == "foo") {
            emit_foo_lint(cx, "a function", item.span());
        }
    }
}
