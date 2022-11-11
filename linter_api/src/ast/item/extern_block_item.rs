use crate::ast::Abi;
use crate::ffi::FfiSlice;

use super::{CommonItemData, ExternalItemKind};

/// An extern block with items like this:
///
/// ```
/// extern "C" {
///     static C_VERSION: i32;
///
///     fn some_c_interface();
/// }
/// ```
///
/// * See <https://doc.rust-lang.org/stable/reference/items/modules.html>
#[repr(C)]
#[derive(Debug)]
pub struct ExternBlockItem<'ast> {
    data: CommonItemData<'ast>,
    abi: Abi,
    items: FfiSlice<'ast, ExternalItemKind<'ast>>,
}

super::impl_item_data!(ExternBlockItem, ExternBlock);

impl<'ast> ExternBlockItem<'ast> {
    pub fn abi(&self) -> Abi {
        self.abi
    }

    pub fn items(&self) -> &[ExternalItemKind<'ast>] {
        self.items.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ExternBlockItem<'ast> {
    pub fn new(data: CommonItemData<'ast>, abi: Abi, items: FfiSlice<'ast, ExternalItemKind<'ast>>) -> Self {
        Self { data, abi, items }
    }
}
