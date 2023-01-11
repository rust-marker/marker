/// This macro marks the given struct as the main [`LintPass`](`crate::LintPass`)
/// for the lint crate. For structs implementing [`Default`] it's enough to only
/// pass in the type. Otherwise, a second argument is required to initialize an
/// instance.
///
/// **Struct initialized with `default()`**
/// ```ignore
/// #[derive(Default)]
/// struct LintPassWithDefault;
/// marker_api::interface::export_lint_pass!(LintPassWithDefault);
/// ```
///
/// **Struct with custom initialization:**
/// ```ignore
/// struct LintPassCustomValue(u32);
/// marker_api::interface::export_lint_pass!(LintPassCustomValue, LintPassCustomValue(3));
/// ```
///
/// This macro will create some hidden items prefixed with two underscores. These
/// are unstable and can change in the future.
///
/// ### Additional notes
///
/// This section provides some additional information which might be useful. Note
/// that this can change in the future.
///
/// The instance is created and stored in a [`thread_local!`]
/// [`RefCell`](`std::cell::RefCell`). One lint crate will always be called by the
/// same thread.
#[macro_export]
macro_rules! export_lint_pass {
    ($pass_ty:ident) => {
        $crate::interface::export_lint_pass!($pass_ty, $pass_ty::default());
    };
    ($pass_ty:ident, $pass_init:expr) => {
        thread_local! {
            #[doc(hidden)]
            static __MARKER_STATE: std::cell::RefCell<$pass_ty> = std::cell::RefCell::new($pass_init);
        }

        #[doc(hidden)]
        mod __marker {
            use $crate::LintPass;

            #[no_mangle]
            pub extern "C" fn get_marker_api_version() -> &'static str {
                $crate::MARKER_API_VERSION
            }

            #[no_mangle]
            pub extern "C" fn set_ast_context<'ast>(cx: &'ast marker_api::context::AstContext<'ast>) {
                $crate::context::set_ast_cx(cx);
            }

            $crate::for_each_lint_pass_fn!($crate::interface::export_lint_pass_fn);
        }
    };
}
pub use export_lint_pass;

/// **!Unstable!**
///
/// This macro is used to generate external functions which can be used to
/// transfer data safely over the C ABI. The counterpart passing the information
/// to here is implemented in `marker_adapter`
#[macro_export]
#[doc(hidden)]
macro_rules! export_lint_pass_fn {
    (fn $fn_name:ident<'ast>(&self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty) => {
        #[no_mangle]
        pub extern "C" fn $fn_name<'ast>($($arg_name: $arg_ty),*) -> $ret_ty {
            super::__MARKER_STATE.with(|state| state.borrow().$fn_name($($arg_name),*))
        }
    };
    (fn $fn_name:ident<'ast>(&(mut) self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty) => {
        #[no_mangle]
        pub extern "C" fn $fn_name<'ast>($($arg_name: $arg_ty),*) -> $ret_ty {
            super::__MARKER_STATE.with(|state| state.borrow_mut().$fn_name($($arg_name),*))
        }
    };
}
pub use export_lint_pass_fn;
