use libloading::Library;
use marker_api::{interface::LintCrateBindings, AstContext};
use marker_api::{LintPass, LintPassInfo, MARKER_API_VERSION};
use std::path::PathBuf;
use thiserror::Error;

use super::{AdapterError, LINT_CRATES_ENV};

/// A struct describing a lint crate that can be loaded
#[derive(Debug, Clone)]
pub struct LintCrateInfo {
    /// The name of the lint crate
    pub name: String,
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
        let env_str = std::env::var_os(LINT_CRATES_ENV).ok_or(AdapterError::LintCratesEnvUnset)?;

        let mut lint_crates = vec![];
        for item in env_str.to_str().ok_or(AdapterError::LintCratesEnvMalformed)?.split(';') {
            let mut item_parts = item.splitn(2, ':');
            let name = item_parts.next().ok_or(AdapterError::LintCratesEnvMalformed)?;
            let path = item_parts.next().ok_or(AdapterError::LintCratesEnvMalformed)?;
            lint_crates.push(LintCrateInfo {
                name: name.to_string(),
                path: PathBuf::from(path),
            });
        }
        Ok(lint_crates)
    }
}

/// This struct loads external lint crates into memory and provides a safe API
/// to call the respective methods on all of them.
#[derive(Debug, Default)]
pub struct LintCrateRegistry {
    passes: Vec<LoadedLintCrate>,
}

impl LintCrateRegistry {
    pub fn new(lint_crates: &[LintCrateInfo]) -> Result<Self, LoadingError> {
        let mut new_self = Self::default();

        for krate in lint_crates {
            new_self.passes.push(LoadedLintCrate::try_from_info(krate.clone())?);
        }

        Ok(new_self)
    }

    pub(super) fn set_ast_context<'ast>(&self, cx: &'ast AstContext<'ast>) {
        for lint_pass in &self.passes {
            (lint_pass.bindings.set_ast_context)(cx);
        }
    }

    pub(crate) fn collect_lint_pass_info(&self) -> Vec<LintPassInfo> {
        self.passes.iter().map(|pass| (pass.bindings.info)()).collect()
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
        let lib: &'static Library = Box::leak(Box::new(unsafe { Library::new(&info.path) }?));

        let pass = LoadedLintCrate::try_from_lib(lib, info)?;

        Ok(pass)
    }

    fn try_from_lib(lib: &'static Library, info: LintCrateInfo) -> Result<Self, LoadingError> {
        // Check API version for verification
        let get_api_version = {
            unsafe {
                lib.get::<unsafe extern "C" fn() -> &'static str>(b"marker_api_version\0")
                    .map_err(|_| LoadingError::MissingApiSymbol)?
            }
        };
        let krate_api_version = unsafe { get_api_version() };
        if krate_api_version != MARKER_API_VERSION {
            return Err(LoadingError::IncompatibleVersion {
                krate_version: krate_api_version.to_string(),
            });
        }

        // Load bindings
        let get_lint_crate_bindings = unsafe {
            lib.get::<extern "C" fn() -> LintCrateBindings>(b"marker_lint_crate_bindings\0")
                .map_err(|_| LoadingError::MissingBindingSymbol)?
        };
        let bindings = get_lint_crate_bindings();

        Ok(Self {
            _lib: lib,
            info,
            bindings,
        })
    }
}

#[derive(Error, Debug)]
pub enum LoadingError {
    #[error("the lint crate could not be loaded: {0:#?}")]
    LibLoading(#[from] libloading::Error),
    #[error("the loaded crate doesn't contain the `marker_api_version` symbol")]
    MissingApiSymbol,
    #[error("the loaded crate doesn't contain the `marker_lint_crate_bindings` symbol")]
    MissingBindingSymbol,
    #[error("incompatible api version:\n- lint-crate api: {krate_version}\n- driver api: {MARKER_API_VERSION}")]
    IncompatibleVersion { krate_version: String },
}
