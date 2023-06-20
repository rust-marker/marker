use crate::ast::generic::{GenericParams, TyParamBound};
use crate::ast::ty::SynTyKind;
use crate::ffi::{FfiOption, FfiSlice};

use super::CommonItemData;

/// A type alias like:
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
    bounds: FfiSlice<'ast, TyParamBound<'ast>>,
    aliased_ty: FfiOption<SynTyKind<'ast>>,
}

super::impl_item_data!(TyAliasItem, TyAlias);

impl<'ast> TyAliasItem<'ast> {
    pub fn generics(&self) -> &GenericParams<'ast> {
        &self.generics
    }

    /// Type aliases in [`TraitItem`](`super::TraitItem`)s can declare bounds on
    /// their types, which implementors needs to follow. This method returns these
    /// bounds. It'll return an empty slice, for type aliases which don't have any
    /// bounds declared.
    pub fn bounds(&self) -> &[TyParamBound<'ast>] {
        self.bounds.get()
    }

    pub fn aliased_ty(&self) -> Option<SynTyKind> {
        self.aliased_ty.copy()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> TyAliasItem<'ast> {
    pub fn new(
        data: CommonItemData<'ast>,
        generics: GenericParams<'ast>,
        bounds: &'ast [TyParamBound<'ast>],
        aliased_ty: Option<SynTyKind<'ast>>,
    ) -> Self {
        Self {
            data,
            generics,
            bounds: bounds.into(),
            aliased_ty: aliased_ty.into(),
        }
    }
}
