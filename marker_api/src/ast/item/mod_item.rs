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
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct ModItem<'ast> {
    data: CommonItemData<'ast>,
    #[cfg_attr(feature = "driver-api", builder(setter(into)))]
    items: FfiSlice<'ast, ItemKind<'ast>>,
}

super::impl_item_data!(ModItem, Mod);

impl<'ast> ModItem<'ast> {
    pub fn items(&self) -> &[ItemKind<'ast>] {
        self.items.get()
    }
}
