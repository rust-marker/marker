use crate::{
    ast::{pat::PatKind, Span, SpanId},
    context::with_cx,
    ffi::{FfiOption, FfiSlice},
};

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

/// A match expression with a scrutinee and [`MatchArm`]s
///
/// ```
/// # let scrutinee: &[i32] = &[1, 2];
/// //    vvvvvvvvv The scrutinee of the expression
/// match scrutinee {
///     // v Arm 0
///     [] => println!("Such much empty"),
///     // v Arm 1
///     [x] if *x == 1 => println!("found a one"),
///     // v Arm 2
///     _ => {
///        // A block as the arm expression
///        println!("default branch");
///     },
/// }
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct MatchExpr<'ast> {
    data: CommonExprData<'ast>,
    scrutinee: ExprKind<'ast>,
    arms: FfiSlice<'ast, MatchArm<'ast>>,
}

impl<'ast> MatchExpr<'ast> {
    pub fn scrutinee(&self) -> ExprKind {
        self.scrutinee
    }

    pub fn arms(&self) -> &[MatchArm<'ast>] {
        self.arms.get()
    }
}

super::impl_expr_data!(MatchExpr<'ast>, Match);

#[cfg(feature = "driver-api")]
impl<'ast> MatchExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, scrutinee: ExprKind<'ast>, arms: &'ast [MatchArm<'ast>]) -> Self {
        Self {
            data,
            scrutinee,
            arms: arms.into(),
        }
    }
}

/// An arm inside a [`MatchExpr`] with an optional guard.
///
/// ```
/// # let scrutinee: &[i32] = &[1, 2];
/// match scrutinee {
/// //  vvvvv A branch with a pattern
///     [] => println!("Such much empty"),
///
/// //  vvv The pattern of the arm
///     [x] if *x == 1 => println!("found a one"),
/// //         ^^^^^^^ The guard expression of the arm
///
/// //   v A wildcard pattern used as a default branch
///      _ => {
///         // A block as the arm expression
///         println!("default branch");
///      },
/// }
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct MatchArm<'ast> {
    span: SpanId,
    pat: PatKind<'ast>,
    guard: FfiOption<ExprKind<'ast>>,
    expr: ExprKind<'ast>,
}

impl<'ast> MatchArm<'ast> {
    pub fn span(&self) -> &Span<'ast> {
        with_cx(self, |cx| cx.get_span(self.span))
    }

    pub fn pat(&self) -> PatKind<'ast> {
        self.pat
    }

    pub fn guard(&self) -> Option<ExprKind<'ast>> {
        self.guard.copy()
    }

    pub fn expr(&self) -> ExprKind<'ast> {
        self.expr
    }

    // FIXME: Add `attrs(&self)` function
}

#[cfg(feature = "driver-api")]
impl<'ast> MatchArm<'ast> {
    pub fn new(span: SpanId, pat: PatKind<'ast>, guard: Option<ExprKind<'ast>>, expr: ExprKind<'ast>) -> Self {
        Self {
            span,
            pat,
            guard: guard.into(),
            expr,
        }
    }
}
