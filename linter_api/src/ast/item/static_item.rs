use crate::ast::{
    ty::{Mutability, Ty},
    BodyId,
};

use super::CommonItemData;

/// ```ignore
/// static mut LEVELS: u32 = 0;
/// // `get_name()` -> `LEVELS`
/// // `get_mutability()` -> _Mutable_
/// // `get_ty()` -> _Ty of u32_
/// // `get_body_id()` -> _BodyId of `0`_
/// ```
#[derive(Debug)]
pub struct StaticItem<'ast> {
    data: CommonItemData<'ast>,
    mutability: Mutability,
    body_id: BodyId,
}

super::impl_item_data!(StaticItem, Static);

impl<'ast> StaticItem<'ast> {
    /// The mutability of this item
    pub fn get_mutability(&self) -> Mutability {
        self.mutability
    }

    /// The defined type of this static item
    pub fn get_ty(&'ast self) -> &'ast dyn Ty<'ast> {
        todo!()
    }

    /// This returns the [`BodyId`] of the initialization body.
    pub fn get_body_id(&self) -> BodyId {
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
