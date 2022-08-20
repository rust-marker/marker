use std::cell::RefCell;

use bumpalo::Bump;
use linter_api::lint::Lint;
use rustc_hash::FxHashMap;

pub struct Storage<'ast> {
    buffer: Bump,
    #[expect(dead_code)]
    lint_map: RefCell<FxHashMap<&'ast Lint, &'static rustc_lint::Lint>>,
}

impl<'ast> Default for Storage<'ast> {
    fn default() -> Self {
        Self {
            buffer: Bump::new(),
            lint_map: RefCell::default(),
        }
    }
}

impl<'ast> Storage<'ast> {
    pub fn alloc<F, T>(&'ast self, f: F) -> &'ast T
    where
        F: FnOnce() -> T,
    {
        self.buffer.alloc_with(f)
    }
}
