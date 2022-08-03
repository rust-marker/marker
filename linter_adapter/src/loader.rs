use std::marker::PhantomData;

use libloading::Library;

use linter_api::ast::item::{ExternCrateItem, ModItem, StaticItem, UseDeclItem};
use linter_api::context::AstContext;
use linter_api::lint::Lint;
use linter_api::LintPass;

#[derive(Default)]
pub struct ExternalLintCrateRegistry<'ast> {
    pub lint_passes: Vec<Box<dyn LintPass<'ast>>>,
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

    pub fn registered_lints(&self) -> Box<[&'static linter_api::lint::Lint]> {
        todo!()
    }
}

struct LoadedLintCrate<'ast> {
    _lib: &'static Library,
    _ast: PhantomData<&'ast ()>,
}

impl<'ast> LoadedLintCrate<'ast> {
    fn try_from_lib(lib: &'static Library) -> Result<Self, LoadingError> {
        let _registered_lints = unsafe {
            lib.get::<unsafe extern "C" fn() -> Box<[&'static Lint]>>(b"registered_lints\0")
                .map_err(|_| LoadingError::MissingLintDeclaration)?
        };

        Ok(Self {
            _lib: lib,
            _ast: PhantomData,
        })
    }
}

// macro_rules! lint_pass_fn_ptr_field {
//     (fn $fn_name:ident(&self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty) => {
//         $fn_name: Symbol<'static, unsafe extern "C" fn($($arg_ty,)*) -> $ret_ty>,
//     };
//     (fn $fn_name:ident(&mut self $(, $arg_name:ident: $arg_ty:ty)*) -> $ret_ty:ty) => {
//         $fn_name: Symbol<'static, unsafe extern "C" fn($($arg_ty,)*) -> $ret_ty>,
//     };
// }
// use lint_pass_fn_ptr_field;

impl<'ast> LintPass<'ast> for ExternalLintCrateRegistry<'ast> {
    fn registered_lints(&self) -> Box<[&'static linter_api::lint::Lint]> {
        Box::new([])
    }

    fn check_item(&mut self, cx: &'ast AstContext<'ast>, item: linter_api::ast::item::ItemType<'ast>) {
        for lint_pass in &mut self.lint_passes {
            lint_pass.check_item(cx, item);
        }
    }

    fn check_mod(&mut self, cx: &'ast AstContext<'ast>, mod_item: &'ast ModItem<'ast>) {
        for lint_pass in &mut self.lint_passes {
            lint_pass.check_mod(cx, mod_item);
        }
    }
    fn check_extern_crate(&mut self, cx: &'ast AstContext<'ast>, extern_crate_item: &'ast ExternCrateItem<'ast>) {
        for lint_pass in &mut self.lint_passes {
            lint_pass.check_extern_crate(cx, extern_crate_item);
        }
    }
    fn check_use_decl(&mut self, cx: &'ast AstContext<'ast>, use_item: &'ast UseDeclItem<'ast>) {
        for lint_pass in &mut self.lint_passes {
            lint_pass.check_use_decl(cx, use_item);
        }
    }

    fn check_static_item(&mut self, cx: &'ast AstContext<'ast>, item: &'ast StaticItem<'ast>) {
        for lint_pass in &mut self.lint_passes {
            lint_pass.check_static_item(cx, item);
        }
    }
}

#[derive(Debug)]
#[expect(dead_code)]
pub enum LoadingError {
    FileNotFound,
    IncompatibleVersion,
    MissingLintDeclaration,
}
