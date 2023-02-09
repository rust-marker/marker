//! This module contains all expressions, which are typically used to construct
//! or deconstruct data. A simple example is the [`ArrayExpr`] which can be
//! used to create or destruct an array.

use crate::ffi::{FfiOption, FfiSlice};

use super::{CommonExprData, ExprKind, ExprPrecedence};

/// An array expressions can be used to construct an array or destruct an array.
///
/// ```
/// //            vvvvvvvvvvvv An array expression with four element expressions
/// let array_1 = [1, 2, 3, 4];
/// //            vvvvvv An array expression with one element and one len expression
/// let array_2 = [6; 3];
///
/// //  vvvvvvvvv An array expression destructing `array_2`
/// let [a, b, c] = array_2;
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct ArrayExpr<'ast> {
    data: CommonExprData<'ast>,
    elements: FfiSlice<'ast, ExprKind<'ast>>,
    len_expr: FfiOption<ExprKind<'ast>>,
}

impl<'ast> ArrayExpr<'ast> {
    pub fn elements(&self) -> &[ExprKind<'ast>] {
        self.elements.get()
    }

    pub fn len_expr(&self) -> Option<ExprKind<'ast>> {
        self.len_expr.copy()
    }
}

super::impl_expr_data!(
    ArrayExpr<'ast>,
    Array,
    fn precedence(&self) -> ExprPrecedence {
        ExprPrecedence::Pattern
    }
);

#[cfg(feature = "driver-api")]
impl<'ast> ArrayExpr<'ast> {
    pub fn new(
        data: CommonExprData<'ast>,
        elem_exprs: &'ast [ExprKind<'ast>],
        len_expr: Option<ExprKind<'ast>>,
    ) -> Self {
        Self {
            data,
            elements: elem_exprs.into(),
            len_expr: len_expr.into(),
        }
    }
}

/// A tuple expression used to construct or deconstruct a tuple.
///
/// ```
/// //          vvvvvvvvvvvv A tuple expression with four elements
/// let slice = (1, 2, 3, 4);
///
/// //  vvvvvvvvvvvv A tuple expression destructing `slice`
/// let (a, b, c, _) = slice;
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct TupleExpr<'ast> {
    data: CommonExprData<'ast>,
    elements: FfiSlice<'ast, ExprKind<'ast>>,
}

impl<'ast> TupleExpr<'ast> {
    pub fn elements(&self) -> &[ExprKind<'ast>] {
        self.elements.get()
    }
}

super::impl_expr_data!(
    TupleExpr<'ast>,
    Tuple,
    fn precedence(&self) -> ExprPrecedence {
        ExprPrecedence::Pattern
    }
);

#[cfg(feature = "driver-api")]
impl<'ast> TupleExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, elements: FfiSlice<'ast, ExprKind<'ast>>) -> Self {
        Self { data, elements }
    }
}
