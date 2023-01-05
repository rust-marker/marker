use crate::ast::SymbolId;
use crate::context::with_cx;

use super::CommonItemData;

/// An extern crate item like:
///
/// ```ignore
/// extern crate std;
/// // `name()`       -> "std"
/// // `crate_name()` -> "std"
/// extern crate std as ruststd;
/// // `name()`       -> "ruststd"
/// // `crate_name()` -> "std"
/// ```
///
/// * See <https://doc.rust-lang.org/stable/reference/items/extern-crates.html>
#[repr(C)]
#[derive(Debug)]
pub struct ExternCrateItem<'ast> {
    data: CommonItemData<'ast>,
    crate_name: SymbolId,
}

super::impl_item_data!(ExternCrateItem, ExternCrate);

impl<'ast> ExternCrateItem<'ast> {
    /// This will return the original name of external crate. This will only differ
    /// with [`ItemData::get_name`](`super::ItemData::name`) if the user has
    /// declared an alias with `as`.
    ///
    /// In most cases, you want to use this over the `get_name()` function.
    pub fn crate_name(&self) -> String {
        with_cx(self, |cx| cx.symbol_str(self.crate_name))
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ExternCrateItem<'ast> {
    pub fn new(data: CommonItemData<'ast>, crate_name: SymbolId) -> Self {
        Self { data, crate_name }
    }
}
