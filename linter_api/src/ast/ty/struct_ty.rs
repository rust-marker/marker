use crate::ast::{item::Generics, DefTyId};

use super::{CommonTyData, FieldDef, VariantKind};

#[repr(C)]
#[derive(Debug)]
pub struct StructTy<'ast> {
    data: CommonTyData<'ast>,
    def_id: DefTyId,
    generics: Generics<'ast>,
    struct_kind: VariantKind<'ast>,
    is_non_exhaustive: bool,
    // FIXME: Add representation/layout info like alignment, size, type
}

#[cfg(feature = "driver-api")]
impl<'ast> StructTy<'ast> {
    pub fn new(
        data: CommonTyData<'ast>,
        def_id: DefTyId,
        generics: Generics<'ast>,
        struct_kind: VariantKind<'ast>,
        is_non_exhaustive: bool,
    ) -> Self {
        Self {
            data,
            def_id,
            generics,
            struct_kind,
            is_non_exhaustive,
        }
    }
}

super::impl_ty_data!(StructTy<'ast>, Struct);

impl<'ast> StructTy<'ast> {
    pub fn def_id(&self) -> DefTyId {
        self.def_id
    }

    pub fn generics(&self) -> &Generics<'ast> {
        &self.generics
    }

    /// Returns `true`, if this is a unit struct like:
    /// ```
    /// struct Name1;
    /// struct Name2 {};
    /// ```
    pub fn is_unit_struct(&self) -> bool {
        matches!(self.struct_kind, VariantKind::Unit)
    }

    /// Returns `true`, if this is a tuple struct like:
    /// ```
    /// struct Name(u32, u64);
    /// ```
    pub fn is_tuple_struct(&self) -> bool {
        matches!(self.struct_kind, VariantKind::Tuple(_))
    }

    /// Returns `true`, if this is a struct with named fields like:
    /// ```
    /// struct Name {
    ///     field: u32,
    /// };
    /// ```
    pub fn is_field_struct(&self) -> bool {
        matches!(self.struct_kind, VariantKind::Field(_))
    }

    pub fn fields(&self) -> &[&FieldDef<'ast>] {
        self.struct_kind.fields()
    }

    pub fn is_non_exhaustive(&self) -> bool {
        self.is_non_exhaustive
    }
}
