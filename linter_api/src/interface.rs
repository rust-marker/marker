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
        #[allow(improper_ctypes_definitions)]
        #[doc(hidden)]
        pub extern "C" fn __register(registry: &mut dyn $crate::interface::LintPassRegistry) {
            registry.register($name, Box::new($lint_pass_instance));
        }
    };
}

pub use export_lint_pass;
