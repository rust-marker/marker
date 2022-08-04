use libloading::Library;

use linter_api::lint::Lint;
use linter_api::LintPass;

#[derive(Default)]
pub struct ExternalLintCrateRegistry<'ast> {
    passes: Vec<LoadedLintCrate<'ast>>,
}

impl<'a> ExternalLintCrateRegistry<'a> {
    /// # Errors
    /// This can return errors if the library couldn't be found or if the
    /// required symbols weren't provided.
    pub fn load_external_lib(&mut self, lib_path: &str) -> Result<(), LoadingError> {
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
    /// Panics if a lint in the environment couln't be loaded.
    pub fn new_from_env() -> Self {
        let mut new_self = Self::default();

        if let Ok(lint_crates_lst) = std::env::var("LINTER_LINT_CRATES") {
            for lint_crate in lint_crates_lst.split(';') {
                if let Err(err) = new_self.load_external_lib(lint_crate) {
                    panic!("Unable to load `{lint_crate}`, reason: {err:?}");
                }
            }
        }

        new_self
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! gen_LoadedLintCrate {
    (($dollar:tt) $(fn $fn_name:ident(& $(($mut_:tt))? self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty;)+) => {
        struct LoadedLintCrate<'ast> {
            _lib: &'static Library,
            $(
                $fn_name: libloading::Symbol<'ast, unsafe extern "C" fn($($arg_ty,)*) -> $ret_ty>,
            )*
        }

        impl<'ast> LoadedLintCrate<'ast> {
            fn try_from_lib(lib: &'static Library) -> Result<Self, LoadingError> {
                // get function pointers
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
                    $(
                        $fn_name,
                    )*
                })
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
linter_api::lint_pass_fns!(crate::gen_LoadedLintCrate);

impl<'ast> LintPass<'ast> for ExternalLintCrateRegistry<'ast> {
    fn registered_lints(&self) -> Box<[&'static Lint]> {
        let mut lints = vec![];
        for lint_pass in self.passes.iter() {
            lints.extend_from_slice(&lint_pass.registered_lints());
        }
        lints.into_boxed_slice()
    }

    linter_api::for_each_lint_pass_fn!(crate::gen_lint_crate_reg_lint_pass_fn);
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

#[derive(Debug)]
#[expect(dead_code)]
pub enum LoadingError {
    FileNotFound,
    IncompatibleVersion,
    MissingLintDeclaration,
}
