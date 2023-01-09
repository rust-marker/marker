use crate::ast::AstPath;

use super::CommonItemData;

/// A `use` declaration like:
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
pub struct UseItem<'ast> {
    data: CommonItemData<'ast>,
    use_path: AstPath<'ast>,
    use_kind: UseKind,
}

super::impl_item_data!(UseItem, Use);

#[repr(C)]
#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
pub(crate) enum UseKind {
    /// Single usages like `use foo::bar` a list of multiple `use` declarations like
    /// `use foo::{bar, baz}` will be desugured to `use foo::bar; use foo::baz;`
    Single,
    /// A glob import like `use foo::*`
    Glob,
}

impl<'ast> UseItem<'ast> {
    /// Returns the path of this `use` item. For blob imports the `*` will
    /// be included in the simple path.
    pub fn use_path(&self) -> &AstPath<'ast> {
        &self.use_path
    }

    pub fn is_glob(&self) -> bool {
        matches!(self.use_kind, UseKind::Glob)
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> UseItem<'ast> {
    pub fn new(data: CommonItemData<'ast>, use_path: AstPath<'ast>, use_kind: UseKind) -> Self {
        Self {
            data,
            use_path,
            use_kind,
        }
    }
}
