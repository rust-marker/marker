use crate::{
    ast::{generic::GenericArgs, SymbolId, TyDefId},
    context::AstContext,
};

use super::{CommonTyData, FieldDef, VariantKind};

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct EnumTy<'ast> {
    data: CommonTyData<'ast>,
    def_id: TyDefId,
    generic_args: GenericArgs<'ast>,
    is_non_exhaustive: bool,
    // FIXME: Add representation/layout info like alignment, size, type
}

#[cfg(feature = "driver-api")]
impl<'ast> EnumTy<'ast> {
    pub fn new(
        data: CommonTyData<'ast>,
        def_id: TyDefId,
        generic_args: GenericArgs<'ast>,
        is_non_exhaustive: bool,
    ) -> Self {
        Self {
            data,
            def_id,
            generic_args,
            is_non_exhaustive,
        }
    }
}

super::impl_ty_data!(EnumTy<'ast>, Enum);

impl<'ast> EnumTy<'ast> {
    pub fn def_id(&self) -> TyDefId {
        self.def_id
    }

    pub fn generic_args(&self) -> &GenericArgs<'ast> {
        &self.generic_args
    }

    pub fn variants(&self) -> &[EnumItem<'ast>] {
        // Add context method to get these, as they are usually not needed
        todo!()
    }

    pub fn is_non_exhaustive(&self) -> bool {
        self.is_non_exhaustive
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct EnumItem<'ast> {
    cx: &'ast AstContext<'ast>,
    def_id: TyDefId,
    name: SymbolId,
    kind: VariantKind<'ast>,
    // FIXME: Add optional expression for variant number
}

#[cfg(feature = "driver-api")]
impl<'ast> EnumItem<'ast> {
    pub fn new(cx: &'ast AstContext<'ast>, def_id: TyDefId, name: SymbolId, kind: VariantKind<'ast>) -> Self {
        Self { cx, def_id, name, kind }
    }
}

impl<'ast> EnumItem<'ast> {
    pub fn def_id(&self) -> TyDefId {
        self.def_id
    }

    pub fn name(&self) -> String {
        self.cx.symbol_str(self.name)
    }

    /// Returns `true`, if this is a unit enum item like:
    /// ```
    /// enum EnumName {
    ///     Item1,
    ///     Item2 {},
    /// }
    /// ```
    pub fn is_unit_item(&self) -> bool {
        matches!(self.kind, VariantKind::Unit)
    }

    /// Returns `true`, if this is a tuple enum item like:
    /// ```
    /// enum EnumName {
    ///     Item1(u32),
    ///     Item2(u16, u16),
    /// }
    /// ```
    pub fn is_tuple_item(&self) -> bool {
        matches!(self.kind, VariantKind::Tuple(_))
    }

    /// Returns `true`, if this is a enum item with named fields like:
    /// ```
    /// enum EnumName {
    ///     Item1 {
    ///         field1: u64,
    ///         field2: u64,
    ///     },
    /// }
    /// ```
    pub fn is_field_item(&self) -> bool {
        matches!(self.kind, VariantKind::Field(_))
    }

    pub fn fields(&self) -> &[FieldDef<'ast>] {
        self.kind.fields()
    }
}
