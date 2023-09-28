use crate::prelude::ExprKind;

use super::CommonPatData;

/// Patterns are used as assignees in [`AssignExpr`](crate::ast::expr::AssignExpr)
/// nodes. Assign expressions can target place expressions like
/// variables, [`IndexExpr`](crate::ast::expr::IndexExpr)s and
/// [`FieldExpr`](crate::ast::expr::FieldExpr)s. These expressions would
/// be stored as this variant.
///
/// ```
/// # fn some_fn() -> (i32, i32) { (4, 5) }
/// # let mut a = 1;
/// # let mut b = (2, 3);
/// # let mut c = [4, 5];
///     a = 6;
/// //  ^ A path expression targeting the local variable `a`
///
///     b.1 = 7;
/// //  ^^^ A field expression accessing field 1 on the local variable `b`
///
///     c[0] = 8;
/// //  ^^^^ An index expression on local variable `c`
///
///     (a, b.0) = some_fn();
/// //   ^  ^^^ Place expressions nested in a tuple pattern
/// ```
///
/// Place expressions can currently only occur as targets in
/// [`AssignExpr`](crate::ast::expr::AssignExpr)s. Patterns from
/// [`LetStmts`](crate::ast::stmt::LetStmt)s and arguments in
/// [`FnItem`](crate::ast::item::FnItem) will never contain place expressions.
/// Static paths identifying [`ConstItem`](crate::ast::item::ConstItem)s or
/// [`EnumItem`](crate::ast::item::EnumItem) variants are expressed with the
/// [`PatKind::Path`](crate::ast::pat::PatKind::Path) variant.
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct PlacePat<'ast> {
    data: CommonPatData<'ast>,
    place: ExprKind<'ast>,
}

impl<'ast> PlacePat<'ast> {
    /// The expression, which identifies the place.
    pub fn place(&self) -> ExprKind<'ast> {
        self.place
    }
}

super::impl_pat_data!(PlacePat<'ast>, Place);
