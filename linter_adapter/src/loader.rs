use libloading::Library;

use linter_api::interface::{LintPassDeclaration, LintPassRegistry};
use linter_api::LintPass;

#[derive(Default)]
pub struct ExternalLintPassRegistry {
    pub lint_passes: Vec<Box<dyn LintPass>>,
    libs: Vec<Library>,
}

impl ExternalLintPassRegistry {
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

        self.libs.push(lib);

        Ok(())
    }
}

impl LintPassRegistry for ExternalLintPassRegistry {
    fn register(&mut self, _name: &str, init: Box<dyn LintPass>) {
        self.lint_passes.push(init);
    }
}

#[derive(Debug)]
pub enum LoadingError {
    FileNotFound,
    IncompatibleVersion,
    MissingLintDeclaration,
}
