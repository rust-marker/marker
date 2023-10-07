use crate::prelude::*;

/// A private super trait, to prevent other creates from implementing Marker's
/// API traits.
///
/// See: [Sealed traits](https://rust-lang.github.io/api-guidelines/future-proofing.html)
pub trait Sealed {}

impl<N: Sealed> Sealed for &N {}

impl Sealed for ast::AssocItemKind<'_> {}
impl Sealed for ast::ClosureParam<'_> {}
impl Sealed for ast::ConstParam<'_> {}
impl Sealed for ast::EnumVariant<'_> {}
impl Sealed for ast::ExprKind<'_> {}
impl Sealed for ast::ExternItemKind<'_> {}
impl Sealed for ast::FnTyParameter<'_> {}
impl Sealed for ast::FnParam<'_> {}
impl Sealed for ast::ItemField<'_> {}
impl Sealed for ast::ItemKind<'_> {}
impl Sealed for ast::LifetimeParam<'_> {}
impl Sealed for ast::LitExprKind<'_> {}
impl Sealed for ast::StmtKind<'_> {}
impl Sealed for ast::StructFieldPat<'_> {}
impl Sealed for ast::TyKind<'_> {}
impl Sealed for ast::TyParam<'_> {}
impl Sealed for Span<'_> {}
impl Sealed for Ident<'_> {}
