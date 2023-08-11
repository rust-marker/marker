use std::marker::PhantomData;

use bumpalo::Bump;

pub struct Storage<'ast> {
    /// The `'ast` lifetime is bound by the `buffer` field. Having it as a parameter
    /// makes it easier to declare it in the return of functions
    _lifetime: PhantomData<&'ast ()>,
    buffer: Bump,
}

impl<'ast> Default for Storage<'ast> {
    fn default() -> Self {
        Self {
            _lifetime: PhantomData,
            buffer: Bump::new(),
        }
    }
}

impl<'ast> Storage<'ast> {
    #[must_use]
    pub fn alloc<T>(&'ast self, t: T) -> &'ast T {
        self.buffer.alloc(t)
    }

    #[must_use]
    pub fn alloc_slice<T, I>(&'ast self, iter: I) -> &'ast [T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        self.buffer.alloc_slice_fill_iter(iter)
    }

    #[must_use]
    pub fn alloc_str(&'ast self, value: &str) -> &'ast str {
        self.buffer.alloc_str(value)
    }
}
