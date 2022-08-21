use std::cell::RefCell;

use bumpalo::Bump;
use linter_api::{
    ast::{item::ItemType, ItemId},
    lint::Lint,
};
use rustc_hash::FxHashMap;

pub struct Storage<'ast> {
    buffer: Bump,
    #[expect(dead_code)]
    lint_map: RefCell<FxHashMap<&'ast Lint, &'static rustc_lint::Lint>>,
    items: RefCell<FxHashMap<ItemId, ItemType<'ast>>>,
}

impl<'ast> Default for Storage<'ast> {
    fn default() -> Self {
        Self {
            buffer: Bump::new(),
            lint_map: RefCell::default(),
            items: RefCell::default(),
        }
    }
}

impl<'ast> Storage<'ast> {
    #[must_use]
    pub fn alloc<F, T>(&'ast self, f: F) -> &'ast T
    where
        F: FnOnce() -> T,
    {
        self.buffer.alloc_with(f)
    }

    #[must_use]
    pub fn alloc_slice_iter<T, I>(&'ast self, iter: I) -> &'ast [T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        self.buffer.alloc_slice_fill_iter(iter)
    }

    pub fn item(&self, id: ItemId) -> Option<ItemType<'ast>> {
        self.items.borrow().get(&id).copied()
    }

    pub fn add_item(&self, id: ItemId, item: ItemType<'ast>) {
        let prev_item = self.items.borrow_mut().insert(id, item);
        debug_assert!(prev_item.is_none(), "no item should ever be mapped and inserted twice");
    }
}
