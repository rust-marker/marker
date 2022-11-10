use crate::ast::generic::GenericParams;
use crate::ast::ty::TyKind;
use crate::ast::{Span, SpanId, SymbolId};
use crate::context::with_cx;
use crate::ffi::FfiSlice;

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
    generics: GenericParams<'ast>,
    fields: FfiSlice<'ast, Field<'ast>>,
}

super::impl_item_data!(UnionItem, Union);

impl<'ast> UnionItem<'ast> {
    pub fn generics(&self) -> &GenericParams<'ast> {
        &self.generics
    }

    pub fn fields(&self) -> &[Field<'ast>] {
        self.fields.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> UnionItem<'ast> {
    pub fn new(data: CommonItemData<'ast>, generics: GenericParams<'ast>, fields: &'ast [Field<'ast>]) -> Self {
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
    generics: GenericParams<'ast>,
    elements: FfiSlice<'ast, EnumElement<'ast>>,
}

super::impl_item_data!(EnumItem, Enum);

impl<'ast> EnumItem<'ast> {
    pub fn generics(&self) -> &GenericParams<'ast> {
        &self.generics
    }

    pub fn elements(&self) -> &[EnumElement<'ast>] {
        self.elements.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> EnumItem<'ast> {
    pub fn new(data: CommonItemData<'ast>, generics: GenericParams<'ast>, elements: &'ast [EnumElement<'ast>]) -> Self {
        Self {
            data,
            generics,
            elements: elements.into(),
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct EnumElement<'ast> {
    ident: SymbolId,
    span: SpanId,
    kind: AdtKind<'ast>,
    // FIXME: Add <discriminant: FfiOption<ExprKind<'ast>>>
}

impl<'ast> EnumElement<'ast> {
    pub fn ident(&self) -> String {
        with_cx(self, |cx| cx.symbol_str(self.ident))
    }

    // FIXME: Add `fn attrs() -> ??? {}`

    /// Returns `true` if this is a union element like:
    ///
    /// ```
    /// pub enum Foo {
    ///     Bar,
    /// }
    /// ```
    pub fn is_union_struct(&self) -> bool {
        matches!(self.kind, AdtKind::Unit)
    }

    /// Returns `true` if this is a tuple element like:
    ///
    /// ```
    /// pub enum Foo {
    ///     Bar(u32, u32)
    /// }
    /// ```
    pub fn is_tuple_element(&self) -> bool {
        matches!(self.kind, AdtKind::Tuple(..))
    }

    /// Returns `true` if this is an element with fields like:
    ///
    /// ```
    /// pub enum Foo {
    ///    Bar {
    ///        data: i32,
    ///        buffer: u32,
    ///    }
    /// }
    /// ```
    pub fn is_field_element(&self) -> bool {
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
        with_cx(self, |cx| cx.get_span(self.span))
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> EnumElement<'ast> {
    pub fn new(ident: SymbolId, span: SpanId, kind: AdtKind<'ast>) -> Self {
        Self { ident, span, kind }
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
    generics: GenericParams<'ast>,
    kind: AdtKind<'ast>,
}

super::impl_item_data!(StructItem, Struct);

impl<'ast> StructItem<'ast> {
    pub fn generics(&self) -> &GenericParams<'ast> {
        &self.generics
    }

    /// Returns `true` if this is a union struct like:
    ///
    /// ```
    /// struct Name1;
    /// struct Name2 {};
    /// ```
    pub fn is_union_struct(&self) -> bool {
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

#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
enum AdtKind<'ast> {
    Unit,
    Tuple(FfiSlice<'ast, Field<'ast>>),
    Field(FfiSlice<'ast, Field<'ast>>),
}

#[repr(C)]
#[derive(Debug)]
pub struct Field<'ast> {
    vis: Visibility<'ast>,
    ident: SymbolId,
    ty: TyKind<'ast>,
    span: SpanId,
}

impl<'ast> Field<'ast> {
    /// The visibility of this item.
    pub fn visibility(&self) -> &Visibility<'ast> {
        &self.vis
    }

    pub fn ident(&self) -> String {
        with_cx(self, |cx| cx.symbol_str(self.ident))
    }

    pub fn ty(&self) -> TyKind<'ast> {
        self.ty
    }

    /// The [`Span`] of the entire item. This span should be used for general item related
    /// diagnostics.
    pub fn span(&self) -> &Span<'ast> {
        with_cx(self, |cx| cx.get_span(self.span))
    }

    // FIXME: Add `fn attrs() -> ??? {}`
}

#[cfg(feature = "driver-api")]
impl<'ast> Field<'ast> {
    pub fn new(vis: Visibility<'ast>, ident: SymbolId, ty: TyKind<'ast>, span: SpanId) -> Self {
        Self { vis, ident, ty, span }
    }
}
