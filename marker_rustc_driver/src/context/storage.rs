use std::cell::RefCell;

use bumpalo::Bump;
use marker_api::ast::SpanSource;
use rustc_hash::FxHashMap;

use crate::conversion::common::SpanSourceInfo;

pub struct Storage<'ast> {
    buffer: Bump,
    span_src_map: RefCell<FxHashMap<rustc_span::FileName, SpanSource<'ast>>>,
    span_infos: RefCell<FxHashMap<SpanSource<'ast>, SpanSourceInfo>>,
}

impl<'ast> Default for Storage<'ast> {
    fn default() -> Self {
        Self {
            buffer: Bump::new(),
            span_src_map: RefCell::default(),
            span_infos: RefCell::default(),
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
    pub fn span_src(&self, rustc_src: &rustc_span::FileName) -> Option<SpanSource<'ast>> {
        self.span_src_map.borrow().get(rustc_src).copied()
    }

    pub fn add_span_src(&self, rustc_src: rustc_span::FileName, api_src: SpanSource<'ast>) {
        let prev_item = self.span_src_map.borrow_mut().insert(rustc_src, api_src);
        debug_assert!(
            prev_item.is_none(),
            "`SpanSource`s should never be mapped and inserted twice"
        );
    }

    pub fn span_src_info(&self, api_src: SpanSource<'ast>) -> Option<SpanSourceInfo> {
        self.span_infos.borrow().get(&api_src).copied()
    }

    pub fn add_span_src_info(&self, api_src: SpanSource<'ast>, src_info: SpanSourceInfo) {
        let prev_item = self.span_infos.borrow_mut().insert(api_src, src_info);
        debug_assert!(
            prev_item.is_none(),
            "`SpanSourceInfo`s should never be mapped and inserted twice"
        );
    }
}