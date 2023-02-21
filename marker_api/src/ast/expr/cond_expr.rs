use crate::{ast::pat::PatKind, ffi::FfiOption};

use super::{CommonExprData, ExprKind};

/// An if expression. If let expressions are expressed as an [`IfExpr`] with an
/// [`LetExpr`] as the conditional expression.
///
/// ```
/// # let cond = true;
/// // vvvv the condition
/// if cond {
///     // then expression
/// } else {
///     // els expression
/// }
///
/// # let slice: &[i32] = &[1, 2];
/// if let [x] = slice {
///     // then expression
/// } // No else expression
///
/// # let num = 5;
/// if num == 1 {
///     // then expression
/// } else /* `IfLet` as an else expression */ if num == 2 {
///     // then expression of the else expression
/// } else {
///     // else expression of the else expression
/// }
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct IfExpr<'ast> {
    data: CommonExprData<'ast>,
    condition: ExprKind<'ast>,
    then: ExprKind<'ast>,
    els: FfiOption<ExprKind<'ast>>,
}

impl<'ast> IfExpr<'ast> {
    pub fn condition(&self) -> ExprKind<'ast> {
        self.condition
    }

    pub fn then(&self) -> ExprKind<'ast> {
        self.then
    }

    pub fn els(&self) -> Option<ExprKind<'ast>> {
        self.els.copy()
    }
}

super::impl_expr_data!(IfExpr<'ast>, If);

#[cfg(feature = "driver-api")]
impl<'ast> IfExpr<'ast> {
    pub fn new(
        data: CommonExprData<'ast>,
        condition: ExprKind<'ast>,
        then: ExprKind<'ast>,
        els: Option<ExprKind<'ast>>,
    ) -> Self {
        Self {
            data,
            condition,
            then,
            els: els.into(),
        }
    }
}

/// A `let` expression used in conditional statements, to check if a pattern
/// matches the scrutinee.
///
/// ```
/// # let slice: &[i32] = &[1, 2];
/// //     vvv The pattern
/// if let [x] = slice {
/// //           ^^^^^ The scrutinee
///     // ...
/// }
///
/// # let mut opt = Some(1);
/// //        vvvvvvv The pattern
/// while let Some(_) = opt {
/// //                  ^^^ The scrutinee
///     // ...
///     # opt = None;
/// }
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct LetExpr<'ast> {
    data: CommonExprData<'ast>,
    pat: PatKind<'ast>,
    scrutinee: ExprKind<'ast>,
}

impl<'ast> LetExpr<'ast> {
    pub fn pat(&self) -> PatKind {
        self.pat
    }

    pub fn scrutinee(&self) -> ExprKind {
        self.scrutinee
    }
}

super::impl_expr_data!(LetExpr<'ast>, Let);

#[cfg(feature = "driver-api")]
impl<'ast> LetExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, pat: PatKind<'ast>, scrutinee: ExprKind<'ast>) -> Self {
        Self { data, pat, scrutinee }
    }
}
