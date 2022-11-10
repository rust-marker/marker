use crate::ast::generic::GenericParams;
use crate::ast::ty::TyKind;
use crate::ffi::FfiOption;

use super::CommonItemData;

/// A type alias like
///
/// ```
/// type Vec3<T: Copy> = (T, T, T);
///
/// trait TraitItem {
///     type AssocType;
/// }
/// ```
///
/// See: <https://doc.rust-lang.org/reference/items/type-aliases.html>
#[repr(C)]
#[derive(Debug)]
pub struct TyAliasItem<'ast> {
    data: CommonItemData<'ast>,
    generics: GenericParams<'ast>,
    aliased_ty: FfiOption<TyKind<'ast>>,
}

super::impl_item_data!(TyAliasItem, TyAlias);

impl<'ast> TyAliasItem<'ast> {
    pub fn generics(&self) -> &GenericParams<'ast> {
        &self.generics
    }

    pub fn aliased_ty(&self) -> Option<TyKind> {
        self.aliased_ty.get().copied()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> TyAliasItem<'ast> {
    pub fn new(data: CommonItemData<'ast>, generics: GenericParams<'ast>, aliased_ty: Option<TyKind<'ast>>) -> Self {
        Self {
            data,
            generics,
            aliased_ty: aliased_ty.into(),
        }
    }
}
