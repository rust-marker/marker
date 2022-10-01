//! FIXME: Should this module remain under `ast::item::` or be moved to `ast::common`

use crate::context::AstContext;

/// A placeholder struct until a proper representation exists.
#[derive(Debug)]
pub struct Generics<'ast> {
    _cx: &'ast AstContext<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> Generics<'ast> {
    #[allow(clippy::used_underscore_binding)]
    pub fn new(_cx: &'ast AstContext<'ast>) -> Self {
        Self { _cx }
    }
}
