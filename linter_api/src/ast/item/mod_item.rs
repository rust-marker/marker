use super::{CommonItemData, ItemType};

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
#[derive(Debug)]
pub struct ModItem<'ast> {
    data: CommonItemData<'ast>,
    items: &'ast [ItemType<'ast>],
}

super::impl_item_data!(ModItem, Mod);

impl<'ast> ModItem<'ast> {
    pub fn get_items(&self) -> &[ItemType<'ast>] {
        self.items
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ModItem<'ast> {
    pub fn new(data: CommonItemData<'ast>, items: &'ast [ItemType<'ast>]) -> Self {
        Self { data, items }
    }
}
