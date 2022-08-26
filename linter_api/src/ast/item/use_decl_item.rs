use crate::ast::ItemPath;

use super::{CommonItemData, UseKind};

/// A use declaration like:
///
/// ```ignore
/// pub use foo::bar::*;
/// // `get_name()`     -> `None`
/// // `get_use_path()` -> `foo::bar::*`
/// // `get_use_kind()` -> `Glob`
/// pub use foo::bar;
/// // `get_name()`     -> `Some(bar)`
/// // `get_use_path()` -> `foo::bar`
/// // `get_use_kind()` -> `Single`
/// pub use foo::bar as baz;
/// // `get_name()`     -> `Some(baz)`
/// // `get_use_path()` -> `foo::bar`
/// // `get_use_kind()` -> `Single`
/// ```
///
/// See <https://doc.rust-lang.org/stable/reference/items/use-declarations.html>
#[derive(Debug)]
pub struct UseDeclItem<'ast> {
    data: CommonItemData<'ast>,
    use_path: ItemPath<'ast>,
    use_kind: UseKind,
}

super::impl_item_data!(UseDeclItem, UseDecl);

impl<'ast> UseDeclItem<'ast> {
    /// Returns the path of this `use` item. For blob imports the `*` will
    /// be included in the simple path.
    pub fn get_use_path(&self) -> &ItemPath<'ast> {
        &self.use_path
    }

    pub fn get_use_kind(&self) -> UseKind {
        self.use_kind
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> UseDeclItem<'ast> {
    pub fn new(data: CommonItemData<'ast>, use_path: ItemPath<'ast>, use_kind: UseKind) -> Self {
        Self {
            data,
            use_path,
            use_kind,
        }
    }
}
