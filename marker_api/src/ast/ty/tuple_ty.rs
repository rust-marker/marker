use crate::ffi::FfiSlice;

use super::{CommonSynTyData, SynTyKind};

#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
pub struct TupleTy<'ast> {
    data: CommonSynTyData<'ast>,
    types: FfiSlice<'ast, SynTyKind<'ast>>,
}

#[cfg(feature = "driver-api")]
impl<'ast> TupleTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, types: &'ast [SynTyKind<'ast>]) -> Self {
        Self {
            data,
            types: types.into(),
        }
    }
}

super::impl_ty_data!(TupleTy<'ast>, Tuple);

impl<'ast> TupleTy<'ast> {
    pub fn types(&self) -> &[SynTyKind<'ast>] {
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
