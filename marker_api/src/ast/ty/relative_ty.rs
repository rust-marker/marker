use crate::{ast::SymbolId, context::with_cx};

use super::{CommonTyData, TyKind};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct RelativeTy<'ast> {
    data: CommonTyData<'ast>,
    ty: TyKind<'ast>,
    name: SymbolId,
}

super::impl_ty_data!(RelativeTy<'ast>, Relative);

impl<'ast> RelativeTy<'ast> {
    /// The type that this type is relative to. This would be the `Iterator` in `Iterator::Item`
    pub fn ty(&self) -> TyKind<'ast> {
        self.ty
    }

    pub fn name(&self) -> &str {
        with_cx(self, |cx| cx.symbol_str(self.name))
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> RelativeTy<'ast> {
    pub fn new(data: CommonTyData<'ast>, ty: TyKind<'ast>, name: SymbolId) -> Self {
        Self { data, ty, name }
    }
}
