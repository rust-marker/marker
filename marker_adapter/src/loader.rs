use crate::error::prelude::*;
use camino::Utf8PathBuf;
use itertools::Itertools;
use libloading::Library;
use marker_api::{LintCrateBindings, MarkerContext};
use marker_api::{LintPass, LintPassInfo, MARKER_API_VERSION};

use super::LINT_CRATES_ENV;

/// A struct describing a lint crate that can be loaded.
#[derive(Debug, Clone)]
pub struct LintCrateInfo {
    /// The name of the lint crate.
    pub name: String,
    /// The absolute path of the compiled dynamic library, which can be loaded as a lint crate.
    pub path: Utf8PathBuf,
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
    pub fn list_from_env() -> Result<Option<Vec<LintCrateInfo>>> {
        let Some(env_str) = std::env::var(LINT_CRATES_ENV).ok() else {
            return Ok(None);
        };

        let mut lint_crates = vec![];
        for item in env_str.split(';') {
            let (name, path) = item.split_once(':').context(|| {
                format!(
                    "The content of the `{LINT_CRATES_ENV}` environment variable is malformed. \
                    Dumped its content on the next line:\n---\n{env_str}\n---",
                )
            })?;

            lint_crates.push(LintCrateInfo {
                name: name.to_string(),
                path: path.into(),
            });
        }
        Ok(Some(lint_crates))
    }
}

/// This struct loads external lint crates into memory and provides a safe API
/// to call the respective methods on all of them.
#[derive(Debug, Default)]
pub struct LintCrateRegistry {
    passes: Vec<LoadedLintCrate>,
}

impl LintCrateRegistry {
    pub fn new(lint_crates: &[LintCrateInfo]) -> Result<Self> {
        let mut new_self = Self::default();

        for krate in lint_crates {
            new_self.passes.push(LoadedLintCrate::try_from_info(krate.clone())?);
        }

        let lint_passes = new_self.collect_lint_pass_info();

        let errors = lint_passes
            .iter()
            .flat_map(LintPassInfo::lints)
            .into_group_map_by(|lint| lint.name.to_ascii_lowercase())
            .into_iter()
            .filter(|(_, lints)| lints.len() > 1)
            .map(|(lint_name, lints)| {
                let defs = lints.iter().map(|lint| format!("- {}", lint.fqn)).format("\n");

                Error::root(format!("The lint `{lint_name}` is defined multiple times:\n{defs}",))
            });

        Error::try_many(errors, "Found several lint name conflicts")?;

        Ok(new_self)
    }

    pub(super) fn set_ast_context<'ast>(&self, cx: &'ast MarkerContext<'ast>) {
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

    fn check_item<'ast>(&mut self, cx: &'ast MarkerContext<'ast>, item: marker_api::ast::ItemKind<'ast>) {
        for lp in &self.passes {
            (lp.bindings.check_item)(cx, item);
        }
    }

    fn check_field<'ast>(&mut self, cx: &'ast MarkerContext<'ast>, field: &'ast marker_api::ast::ItemField<'ast>) {
        for lp in &self.passes {
            (lp.bindings.check_field)(cx, field);
        }
    }

    fn check_variant<'ast>(
        &mut self,
        cx: &'ast MarkerContext<'ast>,
        variant: &'ast marker_api::ast::EnumVariant<'ast>,
    ) {
        for lp in &self.passes {
            (lp.bindings.check_variant)(cx, variant);
        }
    }

    fn check_body<'ast>(&mut self, cx: &'ast MarkerContext<'ast>, body: &'ast marker_api::ast::Body<'ast>) {
        for lp in &self.passes {
            (lp.bindings.check_body)(cx, body);
        }
    }

    fn check_stmt<'ast>(&mut self, cx: &'ast MarkerContext<'ast>, stmt: marker_api::ast::StmtKind<'ast>) {
        for lp in &self.passes {
            (lp.bindings.check_stmt)(cx, stmt);
        }
    }

    fn check_expr<'ast>(&mut self, cx: &'ast MarkerContext<'ast>, expr: marker_api::ast::ExprKind<'ast>) {
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
    fn try_from_info(info: LintCrateInfo) -> Result<Self> {
        let lib = unsafe { Library::new(&info.path) };

        let lib = lib.context(|| format!("Failed to load lint crate `{}`", info.name))?;

        let lib: &'static Library = Box::leak(Box::new(lib));

        let pass = LoadedLintCrate::try_from_lib(lib, info)?;

        Ok(pass)
    }

    fn try_from_lib(lib: &'static Library, info: LintCrateInfo) -> Result<Self> {
        // Check API version for verification
        let get_api_version =
            unsafe { get_symbol::<extern "C" fn() -> &'static str>(lib, &info, b"marker_api_version\0")? };

        let marker_api_version = get_api_version();
        if marker_api_version != MARKER_API_VERSION {
            return Err(Error::from_kind(ErrorKind::IncompatibleMarkerApiVersion {
                lint_krate: info.name,
                marker_api_version: marker_api_version.to_string(),
            }));
        }

        // Load bindings
        let get_lint_crate_bindings =
            unsafe { get_symbol::<extern "C" fn() -> LintCrateBindings>(lib, &info, b"marker_lint_crate_bindings\0")? };

        let bindings = get_lint_crate_bindings();

        Ok(Self {
            _lib: lib,
            info,
            bindings,
        })
    }
}

/// SAFETY: inherits the same safety requirements from [`Library::get`].
unsafe fn get_symbol<T>(
    lib: &'static Library,
    info: &LintCrateInfo,
    symbol_with_nul: &[u8],
) -> Result<libloading::Symbol<'static, T>> {
    lib.get::<T>(symbol_with_nul).context(|| {
        format!(
            "The loaded lint crate {} doesn't contain the symbol {}.\n\
            Dynamic library path: {}",
            info.name,
            // String ignores the trailing nul byte
            String::from_utf8_lossy(symbol_with_nul),
            info.path
        )
    })
}
