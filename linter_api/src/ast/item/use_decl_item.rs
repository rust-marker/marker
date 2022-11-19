use crate::ast::AstPath;

use super::CommonItemData;

/// A use declaration like:
///
/// ```ignore
/// pub use foo::bar::*;
/// // `name()`     -> `None`
/// // `use_path()` -> `foo::bar::*`
/// // `use_kind()` -> `Glob`
/// pub use foo::bar;
/// // `name()`     -> `Some(bar)`
/// // `use_path()` -> `foo::bar`
/// // `use_kind()` -> `Single`
/// pub use foo::bar as baz;
/// // `name()`     -> `Some(baz)`
/// // `use_path()` -> `foo::bar`
/// // `use_kind()` -> `Single`
/// ```
///
/// See <https://doc.rust-lang.org/stable/reference/items/use-declarations.html>
#[repr(C)]
#[derive(Debug)]
pub struct UseDeclItem<'ast> {
    data: CommonItemData<'ast>,
    use_path: AstPath<'ast>,
    use_kind: UseKind,
}

super::impl_item_data!(UseDeclItem, UseDecl);

#[repr(C)]
#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum UseKind {
    /// Single usages like `use foo::bar` a list of multiple usages like
    /// `use foo::{bar, baz}` will be desugured to `use foo::bar; use foo::baz;`
    Single,
    /// A glob import like `use foo::*`
    Glob,
}

impl<'ast> UseDeclItem<'ast> {
    /// Returns the path of this `use` item. For blob imports the `*` will
    /// be included in the simple path.
    pub fn use_path(&self) -> &AstPath<'ast> {
        &self.use_path
    }

    pub fn use_kind(&self) -> UseKind {
        self.use_kind
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> UseDeclItem<'ast> {
    pub fn new(data: CommonItemData<'ast>, use_path: AstPath<'ast>, use_kind: UseKind) -> Self {
        Self {
            data,
            use_path,
            use_kind,
        }
    }
}
