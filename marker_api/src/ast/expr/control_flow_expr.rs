use crate::{
    ast::{pat::PatKind, ExprId, Ident, Span, SpanId},
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
///     // else expression
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
/// } else /* `IfExpr` as an else expression */ if num == 2 {
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

    /// This returns the `else` expression of this `if` expression, this will
    /// either be a [`BlockExpr`](super::BlockExpr) or [`IfExpr`].
    ///
    /// `els` is an abbreviation for `else`, which is a reserved keyword in Rust.
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

/// A `let` expression used in conditional statements, to check if the value
/// of the scrutinee matches the pattern.
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
    pub fn pat(&self) -> PatKind<'ast> {
        self.pat
    }

    pub fn scrutinee(&self) -> ExprKind<'ast> {
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
    pub fn scrutinee(&self) -> ExprKind<'ast> {
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
        with_cx(self, |cx| cx.span(self.span))
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

    // FIXME(xFrednet): Add `fn attrs() -> ??? {}`, see rust-marker/marker#51
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

/// A return expression with an optional value.
///
/// ```
/// pub fn foo(a: bool) {
///     if a {
///         return;
///     //  ^^^^^^ A return expression without a value
///     }
///     // ...
/// }
///
/// pub fn bar(b: bool) -> i32 {
///     if b {
///     //  vvvvvvvvvvvvv A return expression with a value
///         return 0xcafe;
///     //         ^^^^^^ The value of the return expression
///     }
///
///     0xbeef
/// //  ^^^^^^ This is the value of the function body and
/// //         not a return expression
/// }
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct ReturnExpr<'ast> {
    data: CommonExprData<'ast>,
    expr: FfiOption<ExprKind<'ast>>,
}

impl<'ast> ReturnExpr<'ast> {
    pub fn expr(&self) -> Option<ExprKind<'ast>> {
        self.expr.copy()
    }
}

super::impl_expr_data!(ReturnExpr<'ast>, Return);

#[cfg(feature = "driver-api")]
impl<'ast> ReturnExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, expr: Option<ExprKind<'ast>>) -> Self {
        Self {
            data,
            expr: expr.into(),
        }
    }
}

/// A break expression with an optional label and an optional value.
///
/// ```
/// for i in 0..10 {
///     if i == 2 {
///         break;
///     //  ^^^^^ A break expression targeting the for loop
///     }
/// }
///
/// let _ = 'label: {
/// //  vvvvvvvvvvvvvv A break expression with a label and a value
///     break 'label 4;
/// //        ^^^^^^ ^ An integer literal being returned as a value of the target
/// //           |
/// //           An optional label, specifying which labeled expression is the target
/// };
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct BreakExpr<'ast> {
    data: CommonExprData<'ast>,
    label: FfiOption<Ident<'ast>>,
    target_id: ExprId,
    expr: FfiOption<ExprKind<'ast>>,
}

impl<'ast> BreakExpr<'ast> {
    pub fn label(&self) -> Option<&Ident<'ast>> {
        self.label.get()
    }

    pub fn target_id(&self) -> ExprId {
        self.target_id
    }

    pub fn expr(&self) -> Option<ExprKind<'ast>> {
        self.expr.copy()
    }
}

super::impl_expr_data!(BreakExpr<'ast>, Break);

#[cfg(feature = "driver-api")]
impl<'ast> BreakExpr<'ast> {
    pub fn new(
        data: CommonExprData<'ast>,
        label: Option<Ident<'ast>>,
        target_id: ExprId,
        expr: Option<ExprKind<'ast>>,
    ) -> Self {
        Self {
            data,
            label: label.into(),
            target_id,
            expr: expr.into(),
        }
    }
}

/// A continue expression with an optional label.
///
/// ```
/// for i in 0..10 {
///     if i == 2 {
///         continue;
///     //  ^^^^^^^^ A continue expression targeting the for loop
///     }
/// }
///
/// 'label: for a in 0..100 {
///     for b in 0..a {
///         if b == 2 {
///         //  vvvvvvvvvvvvvvv The continue expression targeting the outer loop
///             continue 'label;
///         //           ^^^^^^ The label identifying the target loop
///         }
///         // ...
///     }
/// }
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct ContinueExpr<'ast> {
    data: CommonExprData<'ast>,
    label: FfiOption<Ident<'ast>>,
    target_id: ExprId,
}

impl<'ast> ContinueExpr<'ast> {
    pub fn label(&self) -> Option<&Ident<'ast>> {
        self.label.get()
    }

    pub fn target_id(&self) -> ExprId {
        self.target_id
    }
}

super::impl_expr_data!(ContinueExpr<'ast>, Continue);

#[cfg(feature = "driver-api")]
impl<'ast> ContinueExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, label: Option<Ident<'ast>>, target_id: ExprId) -> Self {
        Self {
            data,
            label: label.into(),
            target_id,
        }
    }
}

/// An unconditional loop expression
///
/// ```
/// # // The `if false` is needed as `cargo test --doc` would not terminate otherwise
/// # if false {
///     //      vvvvvv A loop expression
///     let _ = loop {
///         break 3;
///     //  ^^^^^^^ A break expression targeting the loop and returning a value
///     };
///
/// //  vvvvvvvvvvvvvvvvvv An infinite loop
///     'infinite: loop {};
/// //  ^^^^^^^^^       ^^ A block expression as the loop body expression
/// //      |
/// //      An optional label to be targeted by break and continue expressions
/// # }
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct LoopExpr<'ast> {
    data: CommonExprData<'ast>,
    label: FfiOption<Ident<'ast>>,
    block: ExprKind<'ast>,
}

impl<'ast> LoopExpr<'ast> {
    pub fn label(&self) -> Option<&Ident<'ast>> {
        self.label.get()
    }

    pub fn block(&self) -> ExprKind<'ast> {
        self.block
    }
}

super::impl_expr_data!(LoopExpr<'ast>, Loop);

#[cfg(feature = "driver-api")]
impl<'ast> LoopExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, label: Option<Ident<'ast>>, block: ExprKind<'ast>) -> Self {
        Self {
            data,
            label: label.into(),
            block,
        }
    }
}

/// A `while` loop expression
///
/// ```
///     # let run = false;
/// //  vvvvvvvvvvvv The while loop expression
///     while run {}
/// //        ^^^ ^^ The loop body
/// //         |
/// //         The loop condition
///
///     # let maybe: Option<i32> = None;
/// //  vvvvvv An optional label to be targeted by break and continue expressions
///     'label: while let Some(_) = maybe {}
/// //                ^^^^^^^^^^^^^^^^^^^ A condition using pattern matching
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct WhileExpr<'ast> {
    data: CommonExprData<'ast>,
    label: FfiOption<Ident<'ast>>,
    condition: ExprKind<'ast>,
    block: ExprKind<'ast>,
}

impl<'ast> WhileExpr<'ast> {
    pub fn label(&self) -> Option<&Ident<'ast>> {
        self.label.get()
    }

    pub fn condition(&self) -> ExprKind {
        self.condition
    }

    pub fn block(&self) -> ExprKind<'ast> {
        self.block
    }
}

super::impl_expr_data!(WhileExpr<'ast>, While);

#[cfg(feature = "driver-api")]
impl<'ast> WhileExpr<'ast> {
    pub fn new(
        data: CommonExprData<'ast>,
        label: Option<Ident<'ast>>,
        condition: ExprKind<'ast>,
        block: ExprKind<'ast>,
    ) -> Self {
        Self {
            data,
            condition,
            label: label.into(),
            block,
        }
    }
}

/// A `for` loop iterating over values.
///
/// ```
/// //  vvvvvvvvvvvvvvvvv The for loop expression
///     for i in 0..16 {}
/// //      ^    ^^^^^ A range as the iterable
/// //      |
/// //      A pattern introducing `i` as the iter variable
///
///     # let tuple_iter = [(1, 2)];
/// //  vvvvvv An optional label to be targeted by break and continue expressions
///     'label: for (a, b) in tuple_iter {}
/// //              ^^^^^^ A pattern matching the values of the iterable
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct ForExpr<'ast> {
    data: CommonExprData<'ast>,
    label: FfiOption<Ident<'ast>>,
    pat: PatKind<'ast>,
    iterable: ExprKind<'ast>,
    block: ExprKind<'ast>,
}

impl<'ast> ForExpr<'ast> {
    pub fn label(&self) -> Option<&Ident<'ast>> {
        self.label.get()
    }

    pub fn pat(&self) -> PatKind<'ast> {
        self.pat
    }

    pub fn iterable(&self) -> ExprKind {
        self.iterable
    }

    pub fn block(&self) -> ExprKind<'ast> {
        self.block
    }
}

super::impl_expr_data!(ForExpr<'ast>, For);

#[cfg(feature = "driver-api")]
impl<'ast> ForExpr<'ast> {
    pub fn new(
        data: CommonExprData<'ast>,
        label: Option<Ident<'ast>>,
        pat: PatKind<'ast>,
        iterable: ExprKind<'ast>,
        block: ExprKind<'ast>,
    ) -> Self {
        Self {
            data,
            label: label.into(),
            pat,
            iterable,
            block,
        }
    }
}
