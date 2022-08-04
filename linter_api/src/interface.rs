#[macro_export]
macro_rules! export_lint_pass {
    ($pass_ty:ident) => {
        $crate::interface::export_lint_pass!($pass_ty, $pass_ty::default());
    };
    ($pass_ty:ident, $pass_init:expr) => {
        std::thread_local! {
            #[doc(hidden)]
            static __LINTER_STATE: std::cell::RefCell<$pass_ty> = std::cell::RefCell::new($pass_init);
        }

        #[doc(hidden)]
        mod __linter {
            use $crate::LintPass;

            $crate::for_each_lint_pass_fn!($crate::interface::export_lint_pass_fn);
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! export_lint_pass_fn {
    (fn $fn_name:ident(&self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty) => {
        #[no_mangle]
        pub extern "C" fn $fn_name<'ast>($($arg_name: $arg_ty),*) -> $ret_ty {
            super::__LINTER_STATE.with(|state| state.borrow().$fn_name($($arg_name),*))
        }
    };
    (fn $fn_name:ident(&(mut) self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty) => {
        #[no_mangle]
        pub extern "C" fn $fn_name<'ast>($($arg_name: $arg_ty),*) -> $ret_ty {
            super::__LINTER_STATE.with(|state| state.borrow_mut().$fn_name($($arg_name),*))
        }
    };
}

pub use export_lint_pass;
pub use export_lint_pass_fn;
