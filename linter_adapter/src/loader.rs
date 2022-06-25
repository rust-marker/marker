use libloading::Library;

use linter_api::ast::item::{ExternCrateItem, ModItem, StaticItem, UseDeclItem};
use linter_api::context::AstContext;
use linter_api::interface::{LintPassDeclaration, LintPassRegistry};
use linter_api::LintPass;

#[derive(Default)]
pub struct ExternalLintCrateRegistry<'ast> {
    pub lint_passes: Vec<Box<dyn LintPass<'ast>>>,
    _libs: Vec<Library>,
}

impl<'a> ExternalLintCrateRegistry<'a> {
    /// # Errors
    /// This can return errors if the library couldn't be found or if the
    /// required symbols weren't provided.
    pub fn load_external_lib(&mut self, lib_path: &str) -> Result<(), LoadingError> {
        let lib = unsafe { Library::new(lib_path) }.map_err(|_| LoadingError::FileNotFound)?;

        let decl = unsafe {
            lib.get::<*mut LintPassDeclaration>(b"__lint_pass_declaration\0")
                .map_err(|_| LoadingError::MissingLintDeclaration)?
                .read()
        };

        if decl.linter_api_version != linter_api::LINTER_API_VERSION || decl.rustc_version != linter_api::RUSTC_VERSION
        {
            return Err(LoadingError::IncompatibleVersion);
        }

        unsafe {
            (decl.register)(self);
        }

        self._libs.push(lib);

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

impl<'ast> LintPassRegistry<'ast> for ExternalLintCrateRegistry<'ast> {
    fn register(&mut self, _name: &str, init: Box<dyn LintPass<'ast>>) {
        self.lint_passes.push(init);
    }
}

impl<'ast> LintPass<'ast> for ExternalLintCrateRegistry<'ast> {
    fn registered_lints(&self) -> Vec<&'static linter_api::lint::Lint> {
        let mut all_lints = vec![];
        self.lint_passes
            .iter()
            .for_each(|pass| all_lints.append(&mut pass.registered_lints()));
        all_lints
    }

    fn check_item(&mut self, cx: &'ast AstContext<'ast>, item: linter_api::ast::item::ItemType<'ast>) {
        for lint_pass in self.lint_passes.iter_mut() {
            lint_pass.check_item(cx, item);
        }
    }

    fn check_mod(&mut self, cx: &'ast AstContext<'ast>, mod_item: &'ast ModItem<'ast>) {
        for lint_pass in self.lint_passes.iter_mut() {
            lint_pass.check_mod(cx, mod_item);
        }
    }
    fn check_extern_crate(&mut self, cx: &'ast AstContext<'ast>, extern_crate_item: &'ast ExternCrateItem<'ast>) {
        for lint_pass in self.lint_passes.iter_mut() {
            lint_pass.check_extern_crate(cx, extern_crate_item);
        }
    }
    fn check_use_decl(&mut self, cx: &'ast AstContext<'ast>, use_item: &'ast UseDeclItem<'ast>) {
        for lint_pass in self.lint_passes.iter_mut() {
            lint_pass.check_use_decl(cx, use_item);
        }
    }

    fn check_static_item(&mut self, cx: &'ast AstContext<'ast>, item: &'ast StaticItem<'ast>) {
        for lint_pass in self.lint_passes.iter_mut() {
            lint_pass.check_static_item(cx, item);
        }
    }
}

#[derive(Debug)]
pub enum LoadingError {
    FileNotFound,
    IncompatibleVersion,
    MissingLintDeclaration,
}
