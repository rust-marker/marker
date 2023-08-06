#![doc = include_str!("../README.md")]
#![feature(lint_reasons)]
#![warn(clippy::pedantic)]
#![warn(clippy::index_refutable_slice)]
#![allow(clippy::module_name_repetitions)]

pub mod context;
mod loader;
pub use loader::LintCrateInfo;
use loader::{LintCrateRegistry, LoadingError};

use marker_api::{
    ast::{
        expr::ExprKind,
        item::{Body, EnumVariant, Field, ItemKind},
        stmt::StmtKind,
        Crate,
    },
    context::AstContext,
    LintPass, LintPassInfo,
};
use marker_utils::visitor::{self, Visitor};
use std::{cell::RefCell, ops::ControlFlow};
use thiserror::Error;

pub const LINT_CRATES_ENV: &str = "MARKER_LINT_CRATES";

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("the `{LINT_CRATES_ENV}` environment value is not set")]
    LintCratesEnvUnset,
    /// The format of the environment value is defined in the `README.md` of
    /// the `marker_adapter` crate.
    #[error("the content of the `{LINT_CRATES_ENV}` environment value is malformed")]
    LintCratesEnvMalformed,
    #[error("error while loading the lint crate: {0:#?}")]
    LoadingError(#[from] LoadingError),
}

/// This struct is the interface used by lint drivers to load lint crates, pass
/// `marker_api` objects to external lint passes and all other magic you can think of.
#[derive(Debug)]
pub struct Adapter {
    /// [`LintPass`] functions are called with a mutable `self` parameter as the
    /// first argument. This `RefCell` acts as a wrapper to hide the internal
    /// mutability from drivers.
    ///
    /// The effects of the mutability should never reach the driver anyways and
    /// this just makes it way easier to handle the adapter in drivers.
    inner: RefCell<AdapterInner>,
}

#[derive(Debug)]
struct AdapterInner {
    external_lint_crates: LintCrateRegistry,
}

impl Adapter {
    /// This creates a new [`Adapter`] instance
    ///
    /// # Errors
    ///
    /// This function will return an error if an error occurs during the lint
    /// loading process.
    pub fn new(lint_crates: &[LintCrateInfo]) -> Result<Self, AdapterError> {
        let external_lint_crates = LintCrateRegistry::new(lint_crates)?;
        Ok(Self {
            inner: RefCell::new(AdapterInner { external_lint_crates }),
        })
    }

    #[must_use]
    pub fn lint_pass_infos(&self) -> Vec<LintPassInfo> {
        self.inner.borrow().external_lint_crates.collect_lint_pass_info()
    }

    pub fn process_krate<'ast>(&self, cx: &'ast AstContext<'ast>, krate: &Crate<'ast>) {
        let inner = &mut *self.inner.borrow_mut();

        inner.external_lint_crates.set_ast_context(cx);

        for item in krate.items() {
            visitor::traverse_item::<()>(cx, inner, *item);
        }
    }
}

impl Visitor<()> for AdapterInner {
    fn visit_item<'ast>(&mut self, cx: &'ast AstContext<'ast>, item: ItemKind<'ast>) -> ControlFlow<()> {
        self.external_lint_crates.check_item(cx, item);
        ControlFlow::Continue(())
    }

    fn visit_field<'ast>(&mut self, cx: &'ast AstContext<'ast>, field: &'ast Field<'ast>) -> ControlFlow<()> {
        self.external_lint_crates.check_field(cx, field);
        ControlFlow::Continue(())
    }

    fn visit_variant<'ast>(&mut self, cx: &'ast AstContext<'ast>, variant: &'ast EnumVariant<'ast>) -> ControlFlow<()> {
        self.external_lint_crates.check_variant(cx, variant);
        ControlFlow::Continue(())
    }

    fn visit_body<'ast>(&mut self, cx: &'ast AstContext<'ast>, body: &'ast Body<'ast>) -> ControlFlow<()> {
        self.external_lint_crates.check_body(cx, body);
        ControlFlow::Continue(())
    }

    fn visit_stmt<'ast>(&mut self, cx: &'ast AstContext<'ast>, stmt: StmtKind<'ast>) -> ControlFlow<()> {
        self.external_lint_crates.check_stmt(cx, stmt);
        ControlFlow::Continue(())
    }

    fn visit_expr<'ast>(&mut self, cx: &'ast AstContext<'ast>, expr: ExprKind<'ast>) -> ControlFlow<()> {
        self.external_lint_crates.check_expr(cx, expr);
        ControlFlow::Continue(())
    }
}
