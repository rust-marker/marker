use crate::ffi::FfiSlice;

use super::{CommonItemData, ItemKind};

/// A module item like:
///
/// ```
/// #[allow(clippy::all)] // outer attribute
/// mod module {
///     #![allow(clippy::all)] // inner attribute
///
///     fn item_2() {}
/// }
/// ```
///
/// * See <https://doc.rust-lang.org/stable/reference/items/modules.html>
#[repr(C)]
#[derive(Debug)]
pub struct ModItem<'ast> {
    data: CommonItemData<'ast>,
    items: FfiSlice<'ast, ItemKind<'ast>>,
}

super::impl_item_data!(ModItem, Mod);

impl<'ast> ModItem<'ast> {
    pub fn items(&self) -> &[ItemKind<'ast>] {
        self.items.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ModItem<'ast> {
    pub fn new(data: CommonItemData<'ast>, items: &'ast [ItemKind<'ast>]) -> Self {
        Self {
            data,
            items: items.into(),
        }
    }
}
