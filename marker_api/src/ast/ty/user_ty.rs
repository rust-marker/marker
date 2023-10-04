use crate::ast::AstQPath;

use super::CommonSynTyData;

/// A type identified via a [`AstQPath`]. The kind and definition can be
/// accessed via the ID returned by [`AstQPath::resolve()`].
///
/// A path type is used for:
/// * [Generic types](https://doc.rust-lang.org/reference/items/generics.html#generic-parameters)
/// * [Type aliases](https://doc.rust-lang.org/reference/items/type-aliases.html#type-aliases)
/// * [`Self` types](<https://doc.rust-lang.org/stable/std/keyword.SelfTy.html>)
/// * User defined types like [Structs](https://doc.rust-lang.org/reference/types/struct.html), [Enums](https://doc.rust-lang.org/reference/types/enum.html)
///   and [Unions](https://doc.rust-lang.org/reference/types/union.html)
#[repr(C)]
#[derive(Debug)]
pub struct PathTy<'ast> {
    data: CommonSynTyData<'ast>,
    path: AstQPath<'ast>,
}

impl<'ast> PathTy<'ast> {
    pub fn path(&self) -> &AstQPath<'ast> {
        &self.path
    }
}

super::impl_ty_data!(PathTy<'ast>, Path);

#[cfg(feature = "driver-api")]
impl<'ast> PathTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, path: AstQPath<'ast>) -> Self {
        Self { data, path }
    }
}
