use crate::{common::SymbolId, context::with_cx, ffi::FfiOption};

use super::CommonItemData;

/// A placeholder object for unstable items.
#[repr(C)]
#[derive(Debug)]
pub struct UnstableItem<'ast> {
    data: CommonItemData<'ast>,
    feature: FfiOption<SymbolId>,
}

super::impl_item_data!(UnstableItem, Unstable);

impl<'ast> UnstableItem<'ast> {
    pub fn feature(&self) -> Option<&str> {
        self.feature
            .get()
            .map(|feature| with_cx(self, |cx| cx.symbol_str(*feature)))
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> UnstableItem<'ast> {
    pub fn new(data: CommonItemData<'ast>, feature: Option<SymbolId>) -> Self {
        Self {
            data,
            feature: feature.into(),
        }
    }
}
