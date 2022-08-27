use crate::ast::{ty::Ty, BodyId, Mutability};

use super::CommonItemData;

/// A static item like:
///
/// ```
/// static mut LEVELS: u32 = 0;
/// // `name()` -> `LEVELS`
/// // `is_mutable()` -> true
/// // `ty()` -> _Ty of u32_
/// // `body_id()` -> _BodyId of `0`_
/// ```
///
/// See: <https://doc.rust-lang.org/stable/reference/items/static-items.html>
#[repr(C)]
#[derive(Debug)]
pub struct StaticItem<'ast> {
    data: CommonItemData<'ast>,
    mutability: Mutability,
    body_id: BodyId,
}

super::impl_item_data!(StaticItem, Static);

impl<'ast> StaticItem<'ast> {
    /// The mutability of this item
    pub fn is_mutable(&self) -> bool {
        self.mutability == Mutability::Mut
    }

    /// The defined type of this static item
    pub fn ty(&'ast self) -> &'ast dyn Ty<'ast> {
        #![allow(clippy::missing_panics_doc, clippy::unused_self)]
        todo!()
    }

    /// This returns the [`BodyId`] of the initialization body.
    pub fn body_id(&self) -> BodyId {
        self.body_id
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> StaticItem<'ast> {
    pub fn new(data: CommonItemData<'ast>, mutability: Mutability, body_id: BodyId) -> Self {
        Self {
            data,
            mutability,
            body_id,
        }
    }
}
