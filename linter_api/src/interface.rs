use crate::LintPass;

/// Warning, this is not part of the stable API. It should never be instantiated
/// manually, please use [`export_lint_pass!`] instead.
#[derive(Clone)]
#[repr(C)]
#[doc(hidden)]
pub struct LintPassDeclaration {
    pub linter_api_version: &'static str,
    pub rustc_version: &'static str,
    pub register: unsafe extern "C" fn(&mut dyn LintPassRegistry),
}

pub trait LintPassRegistry<'ast> {
    fn register(&mut self, name: &str, init: Box<dyn LintPass<'ast>>);
}

#[macro_export]
macro_rules! export_lint_pass {
    ($pass_ty:ident) => {
        $crate::interface::export_lint_pass!($pass_ty, $pass_ty :: default());
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
    (fn $fn_name:ident(&mut self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty) => {
        #[no_mangle]
        pub extern "C" fn $fn_name<'ast>($($arg_name: $arg_ty),*) -> $ret_ty {
            super::__LINTER_STATE.with(|state| state.borrow_mut().$fn_name($($arg_name),*))
        }
    };
}


pub use export_lint_pass;
pub use export_lint_pass_fn;

#[macro_export]
macro_rules! export_lint_pass_old {
    ($name:literal, $lint_pass_instance:expr) => {
        #[doc(hidden)]
        #[no_mangle]
        pub static __lint_pass_declaration: $crate::interface::LintPassDeclaration =
            $crate::interface::LintPassDeclaration {
                linter_api_version: $crate::LINTER_API_VERSION,
                rustc_version: $crate::RUSTC_VERSION,
                register: __register,
            };

        /// This function will only be called once it was determined that the
        /// creation was compiled with the same version of rustc. Therefore,
        /// it's safe to pass in an improper type for c.
        ///
        /// It is actually not save, and I have a lot of work left ...
        #[allow(improper_ctypes_definitions)]
        #[doc(hidden)]
        pub extern "C" fn __register(registry: &mut dyn $crate::interface::LintPassRegistry) {
            registry.register($name, Box::new($lint_pass_instance));
        }
    };
}

pub use export_lint_pass_old;
