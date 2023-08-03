use cfg_if::cfg_if;
use libloading::Library;

use marker_api::{interface::LintCrateBindings, AstContext};
use marker_api::{LintPass, LintPassInfo};

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;

use super::{AdapterError, LINT_CRATES_ENV};

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

/// A struct describing a lint crate that can be loaded
#[derive(Debug, Clone)]
pub struct LintCrateInfo {
    /// The absolute path of the compiled dynamic library, which can be loaded as a lint crate.
    pub path: PathBuf,
}

impl LintCrateInfo {
    /// This function tries to load the list of [`LintCrateInfo`]s from the
    /// [`LINT_CRATES_ENV`] environment value.
    ///
    /// # Errors
    ///
    /// This function will return an error if the value can't be read or the
    /// content is malformed. The `README.md` of this adapter contains the
    /// format definition.
    pub fn list_from_env() -> Result<Vec<LintCrateInfo>, AdapterError> {
        let Some(env_str) = std::env::var_os(LINT_CRATES_ENV) else {
            return Err(AdapterError::LintCratesEnvUnset);
        };

        let mut list = vec![];
        for entry_str in split_os_str(&env_str, b';') {
            if entry_str.is_empty() {
                return Err(AdapterError::LintCratesEnvMalformed);
            }

            list.push(LintCrateInfo {
                path: PathBuf::from(entry_str),
            });
        }

        Ok(list)
    }
}

/// This struct loads external lint crates into memory and provides a safe API
/// to call the respective methods on all of them.
#[derive(Debug, Default)]
pub struct LintCrateRegistry {
    passes: Vec<LoadedLintCrate>,
}

impl LintCrateRegistry {
    /// # Panics
    ///
    /// Panics if a lint in the environment couldn't be loaded.
    pub fn new(lint_crates: &[LintCrateInfo]) -> Self {
        let mut new_self = Self::default();

        for krate in lint_crates {
            let lib = match LoadedLintCrate::try_from_info(krate.clone()) {
                Ok(v) => v,
                Err(err) => panic!("Unable to load `{}`, reason: {err:?}", krate.path.display()),
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
    info: LintCrateInfo,
    bindings: LintCrateBindings,
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for LoadedLintCrate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadedLintCrate").field("info", &self.info).finish()
    }
}

impl LoadedLintCrate {
    fn try_from_info(info: LintCrateInfo) -> Result<Self, LoadingError> {
        let lib: &'static Library = Box::leak(Box::new(
            unsafe { Library::new(&info.path) }.map_err(|_| LoadingError::FileNotFound)?,
        ));

        let pass = LoadedLintCrate::try_from_lib(lib, info)?;

        Ok(pass)
    }

    fn try_from_lib(lib: &'static Library, info: LintCrateInfo) -> Result<Self, LoadingError> {
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

        Ok(Self {
            _lib: lib,
            info,
            bindings,
        })
    }
}

#[derive(Debug)]
pub enum LoadingError {
    FileNotFound,
    IncompatibleVersion,
    MissingLintDeclaration,
}
