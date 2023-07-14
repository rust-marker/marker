use std::cell::RefCell;

use bumpalo::Bump;
use marker_api::ast::SpanSource;
use rustc_hash::FxHashMap;

use crate::conversion::common::SpanSourceInfo;

pub struct Storage<'ast> {
    buffer: Bump,
    span_src_info: RefCell<FxHashMap<&'ast SpanSource<'ast>, (&'ast SpanSource<'ast>, SpanSourceInfo)>>,
}

impl<'ast> Default for Storage<'ast> {
    fn default() -> Self {
        Self {
            buffer: Bump::new(),
            span_src_info: RefCell::default(),
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

impl<'ast> Storage<'ast> {
    pub fn get_span_src_info<'a>(
        &'ast self,
        api_src: &'a SpanSource<'a>,
    ) -> Option<(&'ast SpanSource<'ast>, SpanSourceInfo)> {
        self.span_src_info.borrow().get(api_src).copied()
    }

    pub fn insert_span_src_info(
        &'ast self,
        api_src: &SpanSource<'_>,
        src_info: SpanSourceInfo,
    ) -> (&'ast SpanSource<'ast>, SpanSourceInfo) {
        let alloc_api_src = self.alloc(match api_src {
            SpanSource::File(name) => SpanSource::File(self.alloc_str(name.get()).into()),
            SpanSource::Macro(id) => SpanSource::Macro(*id),
            SpanSource::Sugar(name, id) => SpanSource::Sugar(self.alloc_str(name.get()).into(), *id),
        });

        let prev_item = self
            .span_src_info
            .borrow_mut()
            .insert(alloc_api_src, (alloc_api_src, src_info));
        debug_assert!(
            prev_item.is_none(),
            "`SpanSourceInfo`s should never be mapped and inserted twice"
        );

        (alloc_api_src, src_info)
    }
}
