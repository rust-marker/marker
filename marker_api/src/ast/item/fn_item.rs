use crate::ast::generic::GenericParams;
use crate::ast::{impl_callable_data_trait, BodyId, CommonCallableData};
use crate::ffi::FfiOption;

use super::CommonItemData;

/// A function item like:
///
/// ```
/// pub fn foo() {}
///
/// # pub struct SomeItem;
/// impl SomeItem {
///     pub fn bar(&self) {}
/// }
///
/// pub trait SomeTrait {
///     fn baz(_: i32);
/// }
/// ```
///
/// See: <https://doc.rust-lang.org/reference/items/functions.html>
#[repr(C)]
#[derive(Debug)]
pub struct FnItem<'ast> {
    data: CommonItemData<'ast>,
    generics: GenericParams<'ast>,
    callable_data: CommonCallableData<'ast>,
    body_id: FfiOption<BodyId>,
}

super::impl_item_data!(FnItem, Fn);

impl<'ast> FnItem<'ast> {
    pub fn generics(&self) -> &GenericParams<'ast> {
        &self.generics
    }

    pub fn body_id(&self) -> Option<BodyId> {
        self.body_id.copy()
    }
}

impl_callable_data_trait!(FnItem<'ast>);

#[cfg(feature = "driver-api")]
impl<'ast> FnItem<'ast> {
    pub fn new(
        data: CommonItemData<'ast>,
        generics: GenericParams<'ast>,
        callable_data: CommonCallableData<'ast>,
        body: Option<BodyId>,
    ) -> Self {
        Self {
            data,
            generics,
            callable_data,
            body_id: body.into(),
        }
    }
}
