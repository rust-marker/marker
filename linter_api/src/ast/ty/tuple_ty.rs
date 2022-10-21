use crate::ffi::FfiSlice;

use super::{CommonTyData, TyKind};

#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
pub struct TupleTy<'ast> {
    data: CommonTyData<'ast>,
    types: FfiSlice<'ast, TyKind<'ast>>,
}

#[cfg(feature = "driver-api")]
impl<'ast> TupleTy<'ast> {
    pub fn new(data: CommonTyData<'ast>, types: &'ast [TyKind<'ast>]) -> Self {
        Self {
            data,
            types: types.into(),
        }
    }
}

super::impl_ty_data!(TupleTy<'ast>, Tuple);

impl<'ast> TupleTy<'ast> {
    pub fn types(&self) -> &[TyKind<'ast>] {
        self.types.as_slice()
    }
}

impl<'ast> std::fmt::Debug for TupleTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_tuple("");

        for entry in self.types.as_slice() {
            f.field(entry);
        }

        f.finish()
    }
}
