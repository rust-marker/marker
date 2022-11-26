use libloading::Library;

use marker_api::context::AstContext;
use marker_api::lint::Lint;
use marker_api::LintPass;

/// This struct loads external lint crates into memory and provides a safe API
/// to call the respective methods on all of them.
#[derive(Default)]
pub struct LintCrateRegistry<'ast> {
    passes: Vec<LoadedLintCrate<'ast>>,
}

impl<'ast> LintCrateRegistry<'ast> {
    /// # Errors
    /// This can return errors if the library couldn't be found or if the
    /// required symbols weren't provided.
    fn load_external_lib(&mut self, lib_path: &str) -> Result<(), LoadingError> {
        let lib: &'static Library = Box::leak(Box::new(
            unsafe { Library::new(lib_path) }.map_err(|_| LoadingError::FileNotFound)?,
        ));

        let pass = LoadedLintCrate::try_from_lib(lib)?;

        self.passes.push(pass);
        // FIXME: Create issue for lifetimes and fix droping and pointer decl stuff

        Ok(())
    }

    /// # Panics
    ///
    /// Panics if a lint in the environment couldn't be loaded.
    pub fn new_from_env() -> Self {
        let mut new_self = Self::default();

        if let Ok(lint_crates_lst) = std::env::var("MARKER_LINT_CRATES") {
            for lint_crate in lint_crates_lst.split(';') {
                if let Err(err) = new_self.load_external_lib(lint_crate) {
                    panic!("Unable to load `{lint_crate}`, reason: {err:?}");
                }
            }
        }

        new_self
    }

    pub(super) fn set_ast_context(&self, cx: &'ast AstContext<'ast>) {
        for lint_pass in &self.passes {
            lint_pass.set_ast_context(cx);
        }
    }
}

impl<'ast> LintPass<'ast> for LintCrateRegistry<'ast> {
    fn registered_lints(&self) -> Box<[&'static Lint]> {
        let mut lints = vec![];
        for lint_pass in &self.passes {
            lints.extend_from_slice(&lint_pass.registered_lints());
        }
        lints.into_boxed_slice()
    }

    marker_api::for_each_lint_pass_fn!(crate::gen_lint_crate_reg_lint_pass_fn);
}

#[macro_export]
macro_rules! gen_lint_crate_reg_lint_pass_fn {
    (fn $fn_name:ident(&self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty) => {
        // Nothing these will be implemented manually
    };
    (fn $fn_name:ident(&(mut) self $(, $arg_name:ident: $arg_ty:ty)*) -> ()) => {
        fn $fn_name(&mut self $(, $arg_name: $arg_ty)*) {
            for lint_pass in self.passes.iter_mut() {
                lint_pass.$fn_name($($arg_name, )*);
            }
        }
    };
}

/// This macro generates the `LoadedLintCrate` struct, and functions for
/// calling the [`LintPass`] functions. It's the counter part to
/// [`marker_api::interface::export_lint_pass`]
#[macro_export]
macro_rules! gen_LoadedLintCrate {
    (($dollar:tt) $(fn $fn_name:ident(& $(($mut_:tt))? self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty;)+) => {
        /// This struct holds function pointers to api functions in the loaded lint crate
        /// It owns the library instance. It sadly has to be stored as a `'static`
        /// reference due to lifetime restrictions.
        struct LoadedLintCrate<'ast> {
            _lib: &'static Library,
            set_ast_context: libloading::Symbol<'ast, unsafe extern "C" fn(&'ast AstContext<'ast>) -> ()>,
            $(
                $fn_name: libloading::Symbol<'ast, unsafe extern "C" fn($($arg_ty,)*) -> $ret_ty>,
            )*
        }

        impl<'ast> LoadedLintCrate<'ast> {
            /// This function tries to resolve all api functions in the given library.
            fn try_from_lib(lib: &'static Library) -> Result<Self, LoadingError> {
                // get function pointers
                let get_marker_api_version = {
                    unsafe {
                        lib.get::<unsafe extern "C" fn() -> &'static str>(b"get_marker_api_version\0")
                            .map_err(|_| LoadingError::MissingLintDeclaration)?
                    }
                };
                if unsafe { get_marker_api_version() } != marker_api::MARKER_API_VERSION {
                    return Err(LoadingError::IncompatibleVersion);
                }

                let set_ast_context = unsafe {
                    lib.get::<unsafe extern "C" fn(&'ast AstContext<'ast>) -> ()>(b"set_ast_context\0")
                        .map_err(|_| LoadingError::MissingLintDeclaration)?
                };

                $(
                    let $fn_name = {
                        let name: Vec<u8> = stringify!($fn_name).bytes().chain(std::iter::once(b'\0')).collect();
                        unsafe {
                            lib.get::<unsafe extern "C" fn($($arg_ty,)*) -> $ret_ty>(&name)
                                .map_err(|_| LoadingError::MissingLintDeclaration)?
                        }
                    };
                )*
                // create Self
                Ok(Self {
                    _lib: lib,
                    set_ast_context,
                    $(
                        $fn_name,
                    )*
                })
            }

            fn set_ast_context(&self, cx: &'ast AstContext<'ast>) -> () {
                unsafe {
                    (self.set_ast_context)(cx)
                }
            }

            // safe wrapper to external functions
            $(
                fn $fn_name(& $($mut_)* self $(, $arg_name: $arg_ty)*) -> $ret_ty {
                    unsafe {
                        (self.$fn_name)($($arg_name,)*)
                    }
                }
            )*
        }

    };
}
marker_api::lint_pass_fns!(crate::gen_LoadedLintCrate);

#[derive(Debug)]
pub enum LoadingError {
    FileNotFound,
    IncompatibleVersion,
    MissingLintDeclaration,
}
