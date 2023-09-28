use crate::span::Ident;

use super::{CommonExprData, ExprKind};

/// An index expression.
///
/// ```
/// # let slice = &mut [1, 2, 3];
///
/// //          vvvvv the operand of the index expression
///     let _ = slice[0];
/// //                ^ the index expression
///
/// //  vvvvv the operand of the index expression
///     slice[1] = 5;
/// //        ^ the index expression
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct IndexExpr<'ast> {
    data: CommonExprData<'ast>,
    operand: ExprKind<'ast>,
    index: ExprKind<'ast>,
}

impl<'ast> IndexExpr<'ast> {
    pub fn operand(&self) -> ExprKind<'ast> {
        self.operand
    }

    pub fn index(&self) -> ExprKind<'ast> {
        self.index
    }
}

super::impl_expr_data!(IndexExpr<'ast>, Index);

#[cfg(feature = "driver-api")]
impl<'ast> IndexExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, operand: ExprKind<'ast>, index: ExprKind<'ast>) -> Self {
        Self { data, operand, index }
    }
}

/// An expression accessing a field or tuple index.
///
/// ```
/// # #[derive(Default)]
/// # struct FieldStruct {
/// #     a: u32,
/// # }
/// # let mut object = FieldStruct { a: 1 };
/// # let tuple = (1, 2);
/// //          vvvvvv The operand
///     let _ = object.a;
/// //                 ^ The field being accessed
///
/// //  vvvvvv The operand
///     object.a = 2;
/// //         ^ The field being accessed
///
/// //          vvvvv The operand
///     let _ = tuple.0;
/// //                ^ The index of the tuple
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct FieldExpr<'ast> {
    data: CommonExprData<'ast>,
    operand: ExprKind<'ast>,
    field: Ident<'ast>,
}

impl<'ast> FieldExpr<'ast> {
    pub fn operand(&self) -> ExprKind<'ast> {
        self.operand
    }

    pub fn field(&self) -> &Ident<'ast> {
        &self.field
    }
}

super::impl_expr_data!(FieldExpr<'ast>, Field);

#[cfg(feature = "driver-api")]
impl<'ast> FieldExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, operand: ExprKind<'ast>, field: Ident<'ast>) -> Self {
        Self { data, operand, field }
    }
}
