use crate::ast::generic::GenericParams;
use crate::ast::ty::TyKind;
use crate::ast::TraitRef;
use crate::ffi::{FfiOption, FfiSlice};

use super::{AssocItemKind, CommonItemData};

/// An impl item like these examples:
///
/// ```
/// # use core::ops::Add;
/// # struct SomeItem(i32);
/// impl SomeItem {
///     const MAX: i32 = 9;
///
///     pub fn new(data: i32) -> Self {
///         Self(data)
///     }
/// }
///
/// impl Add for SomeItem {
///     type Output = Self;
///
///     fn add(self, other: Self) -> Self {
///         Self::new(self.0 + other.0)
///     }
/// }
///
/// unsafe impl Send for SomeItem {}
/// ```
///
/// * See <https://doc.rust-lang.org/stable/reference/items/modules.html>
#[repr(C)]
#[derive(Debug)]
pub struct ImplItem<'ast> {
    data: CommonItemData<'ast>,
    is_unsafe: bool,
    is_negated: bool,
    trait_ref: FfiOption<TraitRef<'ast>>,
    generics: GenericParams<'ast>,
    ty: TyKind<'ast>,
    items: FfiSlice<'ast, AssocItemKind<'ast>>,
}

super::impl_item_data!(ImplItem, Impl);

impl<'ast> ImplItem<'ast> {
    pub fn is_unsafe(&self) -> bool {
        self.is_unsafe
    }

    pub fn generics(&self) -> &GenericParams<'ast> {
        &self.generics
    }

    pub fn is_negated(&self) -> bool {
        self.is_negated
    }

    pub fn is_trait_impl(&self) -> bool {
        matches!(self.trait_ref, FfiOption::Some(..))
    }

    pub fn trait_ref(&self) -> Option<&TraitRef<'ast>> {
        self.trait_ref.get()
    }

    pub fn items(&self) -> &[AssocItemKind<'ast>] {
        self.items.get()
    }

    pub fn ty(&self) -> TyKind {
        self.ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ImplItem<'ast> {
    pub fn new(
        data: CommonItemData<'ast>,
        is_unsafe: bool,
        is_negated: bool,
        trait_ref: Option<TraitRef<'ast>>,
        generics: GenericParams<'ast>,
        ty: TyKind<'ast>,
        items: &'ast [AssocItemKind<'ast>],
    ) -> Self {
        Self {
            data,
            is_unsafe,
            is_negated,
            trait_ref: trait_ref.into(),
            generics,
            ty,
            items: items.into(),
        }
    }
}
