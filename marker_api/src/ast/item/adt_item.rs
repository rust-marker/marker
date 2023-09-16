use crate::ast::expr::ConstExpr;
use crate::ast::generic::SynGenericParams;
use crate::ast::ty::SynTyKind;
use crate::ast::{FieldId, Span, SpanId, SymbolId, VariantId};
use crate::context::with_cx;
use crate::ffi::{FfiOption, FfiSlice};

use super::{CommonItemData, Visibility};

/// A union item like:
///
/// ```
/// pub union Foo {
///     a: i32,
///     b: f32,
/// }
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct UnionItem<'ast> {
    data: CommonItemData<'ast>,
    generics: SynGenericParams<'ast>,
    fields: FfiSlice<'ast, Field<'ast>>,
}

super::impl_item_data!(UnionItem, Union);

impl<'ast> UnionItem<'ast> {
    pub fn generics(&self) -> &SynGenericParams<'ast> {
        &self.generics
    }

    pub fn fields(&self) -> &[Field<'ast>] {
        self.fields.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> UnionItem<'ast> {
    pub fn new(data: CommonItemData<'ast>, generics: SynGenericParams<'ast>, fields: &'ast [Field<'ast>]) -> Self {
        Self {
            data,
            generics,
            fields: fields.into(),
        }
    }
}

/// An enum item like:
///
/// ```
/// #[repr(u32)]
/// pub enum Foo {
///     Elem1,
///     Elem2 = 1,
///     Elem3(u32),
///     Elem4 {
///         field_1: u32,
///         field_2: u32,
///     }
/// }
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct EnumItem<'ast> {
    data: CommonItemData<'ast>,
    generics: SynGenericParams<'ast>,
    variants: FfiSlice<'ast, EnumVariant<'ast>>,
}

super::impl_item_data!(EnumItem, Enum);

impl<'ast> EnumItem<'ast> {
    pub fn generics(&self) -> &SynGenericParams<'ast> {
        &self.generics
    }

    pub fn variants(&self) -> &[EnumVariant<'ast>] {
        self.variants.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> EnumItem<'ast> {
    pub fn new(
        data: CommonItemData<'ast>,
        generics: SynGenericParams<'ast>,
        variants: &'ast [EnumVariant<'ast>],
    ) -> Self {
        Self {
            data,
            generics,
            variants: variants.into(),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct EnumVariant<'ast> {
    id: VariantId,
    ident: SymbolId,
    span: SpanId,
    kind: AdtKind<'ast>,
    discriminant: FfiOption<ConstExpr<'ast>>,
}

impl<'ast> EnumVariant<'ast> {
    pub fn id(&self) -> VariantId {
        self.id
    }

    pub fn ident(&self) -> &str {
        with_cx(self, |cx| cx.symbol_str(self.ident))
    }

    // FIXME(xFrednet): Add `fn attrs() -> ??? {}`, see rust-marker/marker#51

    /// Returns `true` if this is a unit variant like:
    ///
    /// ```
    /// pub enum Foo {
    ///     Bar,
    /// }
    /// ```
    pub fn is_unit_variant(&self) -> bool {
        matches!(self.kind, AdtKind::Unit)
    }

    /// Returns `true` if this is a tuple variant like:
    ///
    /// ```
    /// pub enum Foo {
    ///     Bar(u32, u32)
    /// }
    /// ```
    pub fn is_tuple_variant(&self) -> bool {
        matches!(self.kind, AdtKind::Tuple(..))
    }

    /// Returns `true` if this is an variant with fields like:
    ///
    /// ```
    /// pub enum Foo {
    ///    Bar {
    ///        data: i32,
    ///        buffer: u32,
    ///    }
    /// }
    /// ```
    pub fn is_field_variant(&self) -> bool {
        matches!(self.kind, AdtKind::Field(..))
    }

    pub fn fields(&self) -> &[Field<'ast>] {
        match &self.kind {
            AdtKind::Unit => &[],
            AdtKind::Tuple(fields) | AdtKind::Field(fields) => fields.get(),
        }
    }

    /// The [`Span`] of the entire item. This span should be used for general item related
    /// diagnostics.
    pub fn span(&self) -> &Span<'ast> {
        with_cx(self, |cx| cx.span(self.span))
    }

    /// The discriminant of this variant, if one has been defined
    pub fn discriminant(&self) -> Option<&ConstExpr<'ast>> {
        self.discriminant.get()
    }
}

crate::diagnostic::impl_emission_node_for_node!(&EnumVariant<'ast>);

#[cfg(feature = "driver-api")]
impl<'ast> EnumVariant<'ast> {
    pub fn new(
        id: VariantId,
        ident: SymbolId,
        span: SpanId,
        kind: AdtKind<'ast>,
        discriminant: Option<ConstExpr<'ast>>,
    ) -> Self {
        Self {
            id,
            ident,
            span,
            kind,
            discriminant: discriminant.into(),
        }
    }
}

/// A struct item like:
///
/// ```
/// pub struct Foo;
/// pub struct Bar(u32, u32);
/// pub struct Baz {
///     field_1: u32,
///     field_2: u32,
/// }
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct StructItem<'ast> {
    data: CommonItemData<'ast>,
    generics: SynGenericParams<'ast>,
    kind: AdtKind<'ast>,
}

super::impl_item_data!(StructItem, Struct);

impl<'ast> StructItem<'ast> {
    pub fn generics(&self) -> &SynGenericParams<'ast> {
        &self.generics
    }

    /// Returns `true` if this is a unit struct like:
    ///
    /// ```
    /// struct Name1;
    /// struct Name2 {};
    /// ```
    pub fn is_unit_struct(&self) -> bool {
        matches!(self.kind, AdtKind::Unit)
    }

    /// Returns `true` if this is a tuple struct like:
    ///
    /// ```
    /// struct Name(u32, u64);
    /// ```
    pub fn is_tuple_struct(&self) -> bool {
        matches!(self.kind, AdtKind::Tuple(..))
    }

    /// Returns `true` if this is a field struct like:
    ///
    /// ```
    /// struct Name {
    ///     field: u32,
    /// };
    /// ```
    pub fn is_field_struct(&self) -> bool {
        matches!(self.kind, AdtKind::Field(..))
    }

    pub fn fields(&self) -> &[Field<'ast>] {
        match &self.kind {
            AdtKind::Unit => &[],
            AdtKind::Tuple(fields) | AdtKind::Field(fields) => fields.get(),
        }
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> StructItem<'ast> {
    pub fn new(data: CommonItemData<'ast>, generics: SynGenericParams<'ast>, kind: AdtKind<'ast>) -> Self {
        Self { data, generics, kind }
    }
}

#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
enum AdtKind<'ast> {
    Unit,
    Tuple(FfiSlice<'ast, Field<'ast>>),
    Field(FfiSlice<'ast, Field<'ast>>),
}

impl<'ast> AdtKind<'ast> {
    // The slice lifetime here is explicitly denoted, as this is used by the
    // driver for convenience and is not part of the public API
    pub fn fields(self) -> &'ast [Field<'ast>] {
        match self {
            AdtKind::Tuple(fields) | AdtKind::Field(fields) => fields.get(),
            AdtKind::Unit => &[],
        }
    }
}

/// A single field inside a [`StructItem`] or [`UnionItem`] with an identifier
/// type and span.
#[repr(C)]
#[derive(Debug)]
pub struct Field<'ast> {
    id: FieldId,
    vis: Visibility<'ast>,
    ident: SymbolId,
    ty: SynTyKind<'ast>,
    span: SpanId,
}

impl<'ast> Field<'ast> {
    pub fn id(&self) -> FieldId {
        self.id
    }

    /// The [`Visibility`] of this item.
    pub fn visibility(&self) -> &Visibility<'ast> {
        &self.vis
    }

    pub fn ident(&self) -> &str {
        with_cx(self, |cx| cx.symbol_str(self.ident))
    }

    pub fn ty(&self) -> SynTyKind<'ast> {
        self.ty
    }

    /// The [`Span`] of the entire item. This span should be used for general item related
    /// diagnostics.
    pub fn span(&self) -> &Span<'ast> {
        with_cx(self, |cx| cx.span(self.span))
    }

    // FIXME(xFrednet): Add `fn attrs() -> ??? {}`, see rust-marker/marker#51
}

crate::diagnostic::impl_emission_node_for_node!(&Field<'ast>);

#[cfg(feature = "driver-api")]
impl<'ast> Field<'ast> {
    pub fn new(id: FieldId, vis: Visibility<'ast>, ident: SymbolId, ty: SynTyKind<'ast>, span: SpanId) -> Self {
        Self {
            id,
            vis,
            ident,
            ty,
            span,
        }
    }
}
