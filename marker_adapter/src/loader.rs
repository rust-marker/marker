use cfg_if::cfg_if;
use libloading::Library;

use marker_api::{interface::LintCrateBindings, AstContext};
use marker_api::{LintPass, LintPassInfo};

use std::ffi::{OsStr, OsString};

/// Splits [`OsStr`] by an ascii character
fn split_os_str(s: &OsStr, c: u8) -> Vec<OsString> {
    cfg_if! {
        if #[cfg(unix)] {
            unix_split_os_str(s, c)
        } else if #[cfg(windows)] {
            windows_split_os_str(s, c)
        } else {
            unimplemented!("`split_os_str` currently works only on unix and windows")
        }
    }
}

#[cfg(unix)]
#[doc(hidden)]
fn unix_split_os_str(s: &OsStr, c: u8) -> Vec<OsString> {
    use std::os::unix::ffi::OsStrExt;

    s.as_bytes()
        .split(|byte| *byte == c)
        .map(|bytes| OsStr::from_bytes(bytes).into())
        .collect()
}

#[cfg(windows)]
#[doc(hidden)]
fn windows_split_os_str(s: &OsStr, c: u8) -> Vec<OsString> {
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    let bytes: Vec<u16> = s.encode_wide().collect();

    bytes.split(|v| *v == u16::from(c)).map(OsString::from_wide).collect()
}

/// This struct loads external lint crates into memory and provides a safe API
/// to call the respective methods on all of them.
#[derive(Default)]
pub struct LintCrateRegistry {
    passes: Vec<LoadedLintCrate>,
}

impl LintCrateRegistry {
    /// # Errors
    /// This can return errors if the library couldn't be found or if the
    /// required symbols weren't provided.
    fn load_external_lib(lib_path: &OsStr) -> Result<LoadedLintCrate, LoadingError> {
        let lib: &'static Library = Box::leak(Box::new(
            unsafe { Library::new(lib_path) }.map_err(|_| LoadingError::FileNotFound)?,
        ));

        let pass = LoadedLintCrate::try_from_lib(lib)?;

        // FIXME: Create issue for lifetimes and fix droping and pointer decl stuff

        Ok(pass)
    }

    /// # Panics
    ///
    /// Panics if a lint in the environment couldn't be loaded.
    pub fn new_from_env() -> Self {
        let mut new_self = Self::default();

        let Some((_, lint_crates_lst)) = std::env::vars_os().find(|(name, _val)| name == "MARKER_LINT_CRATES") else {
            panic!("Adapter tried to find `MARKER_LINT_CRATES` env variable, but it was not present");
        };

        for lib in split_os_str(&lint_crates_lst, b';') {
            if lib.is_empty() {
                continue;
            }

            let lib = match Self::load_external_lib(&lib) {
                Ok(v) => v,
                Err(err) => panic!("Unable to load `{}`, reason: {err:?}", lib.to_string_lossy()),
            };

            new_self.passes.push(lib);
        }

        new_self
    }

    pub(super) fn set_ast_context<'ast>(&self, cx: &'ast AstContext<'ast>) {
        for lint_pass in &self.passes {
            (lint_pass.bindings.set_ast_context)(cx);
        }
    }

    pub(crate) fn collect_lint_pass_info(&self) -> Vec<LintPassInfo> {
        let mut info = vec![];
        for pass in &self.passes {
            info.push((pass.bindings.info)());
        }
        info
    }
}

#[warn(clippy::missing_trait_methods)]
impl LintPass for LintCrateRegistry {
    fn info(&self) -> LintPassInfo {
        panic!("`registered_lints` should not be called on `LintCrateRegistry`");
    }

    fn check_item<'ast>(&mut self, cx: &'ast AstContext<'ast>, item: marker_api::ast::item::ItemKind<'ast>) {
        for lp in &self.passes {
            (lp.bindings.check_item)(cx, item);
        }
    }

    fn check_field<'ast>(&mut self, cx: &'ast AstContext<'ast>, field: &'ast marker_api::ast::item::Field<'ast>) {
        for lp in &self.passes {
            (lp.bindings.check_field)(cx, field);
        }
    }

    fn check_variant<'ast>(
        &mut self,
        cx: &'ast AstContext<'ast>,
        variant: &'ast marker_api::ast::item::EnumVariant<'ast>,
    ) {
        for lp in &self.passes {
            (lp.bindings.check_variant)(cx, variant);
        }
    }

    fn check_body<'ast>(&mut self, cx: &'ast AstContext<'ast>, body: &'ast marker_api::ast::item::Body<'ast>) {
        for lp in &self.passes {
            (lp.bindings.check_body)(cx, body);
        }
    }

    fn check_stmt<'ast>(&mut self, cx: &'ast AstContext<'ast>, stmt: marker_api::ast::stmt::StmtKind<'ast>) {
        for lp in &self.passes {
            (lp.bindings.check_stmt)(cx, stmt);
        }
    }

    fn check_expr<'ast>(&mut self, cx: &'ast AstContext<'ast>, expr: marker_api::ast::expr::ExprKind<'ast>) {
        for lp in &self.passes {
            (lp.bindings.check_expr)(cx, expr);
        }
    }
}

struct LoadedLintCrate {
    _lib: &'static Library,
    bindings: LintCrateBindings,
}

impl LoadedLintCrate {
    fn try_from_lib(lib: &'static Library) -> Result<Self, LoadingError> {
        // Check API version for verification
        let get_api_version = {
            unsafe {
                lib.get::<unsafe extern "C" fn() -> &'static str>(b"marker_api_version\0")
                    .map_err(|_| LoadingError::MissingLintDeclaration)?
            }
        };
        if unsafe { get_api_version() } != marker_api::MARKER_API_VERSION {
            return Err(LoadingError::IncompatibleVersion);
        }

        // Load bindings
        let get_lint_crate_bindings = unsafe {
            lib.get::<extern "C" fn() -> LintCrateBindings>(b"marker_lint_crate_bindings\0")
                .map_err(|_| LoadingError::MissingLintDeclaration)?
        };
        let bindings = get_lint_crate_bindings();

        Ok(Self { _lib: lib, bindings })
    }
}

#[derive(Debug)]
pub enum LoadingError {
    FileNotFound,
    IncompatibleVersion,
    MissingLintDeclaration,
}
