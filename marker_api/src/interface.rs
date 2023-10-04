//! A module responsible for generating and exposing an interface from lint crates.
//! [`export_lint_pass`](crate::export_lint_pass) is the main macro, from this module.

use crate::{context::MarkerContext, ffi::FfiSlice, lint::Lint};

/// **!Unstable!**
/// This struct is used to connect lint crates to drivers.
#[repr(C)]
#[doc(hidden)]
pub struct LintCrateBindings {
    pub set_ast_context: for<'ast> extern "C" fn(cx: &'ast MarkerContext<'ast>),

    // lint pass functions
    pub info: for<'ast> extern "C" fn() -> LintPassInfo,
    pub check_item: for<'ast> extern "C" fn(&'ast MarkerContext<'ast>, crate::ast::ItemKind<'ast>),
    pub check_field: for<'ast> extern "C" fn(&'ast MarkerContext<'ast>, &'ast crate::ast::ItemField<'ast>),
    pub check_variant: for<'ast> extern "C" fn(&'ast MarkerContext<'ast>, &'ast crate::ast::EnumVariant<'ast>),
    pub check_body: for<'ast> extern "C" fn(&'ast MarkerContext<'ast>, &'ast crate::ast::Body<'ast>),
    pub check_stmt: for<'ast> extern "C" fn(&'ast MarkerContext<'ast>, crate::ast::StmtKind<'ast>),
    pub check_expr: for<'ast> extern "C" fn(&'ast MarkerContext<'ast>, crate::ast::ExprKind<'ast>),
}

/// This macro marks the given struct as the main [`LintPass`](`crate::LintPass`)
/// for the lint crate. For structs implementing [`Default`] it's enough to only
/// pass in the type. Otherwise, a second argument is required to initialize an
/// instance.
///
/// **Struct initialized with `default()`**
/// ```ignore
/// #[derive(Default)]
/// struct LintPassWithDefault;
/// marker_api::export_lint_pass!(LintPassWithDefault);
/// ```
///
/// **Struct with custom initialization:**
/// ```ignore
/// struct LintPassCustomValue {
///     // ...
/// };
/// marker_api::export_lint_pass!(LintPassCustomValue, LintPassCustomValue::new(/* ... */));
/// ```
///
/// This macro will create some hidden items prefixed with two underscores. These
/// are unstable and can change in the future.
///
/// #### Driver information
/// * Rustc's driver will always call lint pass instance with the same thread
/// * Rustc's driver will create a new instance for every crate that is being checked
#[macro_export]
macro_rules! export_lint_pass {
    ($pass_ty:ident) => {
        $crate::export_lint_pass!($pass_ty, $pass_ty::default());
    };
    ($pass_ty:ident, $pass_init:expr) => {
        thread_local! {
            #[doc(hidden)]
            static __MARKER_STATE: std::cell::RefCell<$pass_ty> = std::cell::RefCell::new($pass_init);
        }

        #[doc(hidden)]
        mod __marker_todo {
            use $crate::LintPass;

            #[no_mangle]
            extern "C" fn marker_api_version() -> &'static str {
                $crate::MARKER_API_VERSION
            }

            /// This magic function fills the `LintCrateBindings` struct to allow easy
            /// communication between marker's driver and lint crates.
            #[no_mangle]
            extern "C" fn marker_lint_crate_bindings() -> $crate::LintCrateBindings {
                pub use $crate::LintPass;

                extern "C" fn set_ast_context<'ast>(cx: &'ast $crate::MarkerContext<'ast>) {
                    $crate::context::set_ast_cx(cx);
                }
                extern "C" fn info() -> $crate::LintPassInfo {
                    super::__MARKER_STATE.with(|state| state.borrow_mut().info())
                }
                extern "C" fn check_item<'ast>(
                    cx: &'ast $crate::MarkerContext<'ast>,
                    item: $crate::ast::ItemKind<'ast>,
                ) {
                    super::__MARKER_STATE.with(|state| state.borrow_mut().check_item(cx, item));
                }
                extern "C" fn check_field<'ast>(
                    cx: &'ast $crate::MarkerContext<'ast>,
                    field: &'ast $crate::ast::ItemField<'ast>,
                ) {
                    super::__MARKER_STATE.with(|state| state.borrow_mut().check_field(cx, field));
                }
                extern "C" fn check_variant<'ast>(
                    cx: &'ast $crate::MarkerContext<'ast>,
                    variant: &'ast $crate::ast::EnumVariant<'ast>,
                ) {
                    super::__MARKER_STATE.with(|state| state.borrow_mut().check_variant(cx, variant));
                }
                extern "C" fn check_body<'ast>(
                    cx: &'ast $crate::MarkerContext<'ast>,
                    body: &'ast $crate::ast::Body<'ast>,
                ) {
                    super::__MARKER_STATE.with(|state| state.borrow_mut().check_body(cx, body));
                }
                extern "C" fn check_stmt<'ast>(
                    cx: &'ast $crate::MarkerContext<'ast>,
                    stmt: $crate::ast::StmtKind<'ast>,
                ) {
                    super::__MARKER_STATE.with(|state| state.borrow_mut().check_stmt(cx, stmt));
                }
                extern "C" fn check_expr<'ast>(
                    cx: &'ast $crate::MarkerContext<'ast>,
                    expr: $crate::ast::ExprKind<'ast>,
                ) {
                    super::__MARKER_STATE.with(|state| state.borrow_mut().check_expr(cx, expr));
                }

                $crate::LintCrateBindings {
                    set_ast_context,
                    info,
                    check_item,
                    check_field,
                    check_variant,
                    check_body,
                    check_stmt,
                    check_expr,
                }
            }
        }
    };
}

#[derive(Debug)]
pub struct LintPassInfoBuilder {
    lints: &'static [&'static Lint],
}

impl LintPassInfoBuilder {
    /// This method creates a new [`LintPassInfoBuilder`] with minimal required information.
    ///
    /// The `lints` argument should contain all lints which can be emitted by this crate. It
    /// allows the driver to track the lint level.
    pub fn new(lints: Box<[&'static Lint]>) -> Self {
        Self {
            // It's hard to add lifetimes to the `LintPassInfo` due to how and when it
            // is called. Ideally, it would be cool to just store the `Box` directly but
            // that is sadly not possible due to ABI constraints
            lints: Box::leak(lints),
        }
    }

    /// This method builds the [`LintPassInfo`], ready for consumption.
    pub fn build(self) -> LintPassInfo {
        LintPassInfo {
            lints: self.lints.into(),
        }
    }
}

/// This struct provides basic information required by the driver. It can also
/// be used to provide additional information. The struct is constructed using
/// the [`LintPassInfoBuilder`].
///
/// All references and pointers in this struct have to have the `'static` lifetime
/// due to ABI constraints.
#[repr(C)]
#[derive(Debug)]
#[non_exhaustive]
pub struct LintPassInfo {
    lints: FfiSlice<'static, &'static Lint>,
}

#[cfg(feature = "driver-api")]
impl LintPassInfo {
    pub fn lints(&self) -> &[&'static Lint] {
        self.lints.get()
    }
}
