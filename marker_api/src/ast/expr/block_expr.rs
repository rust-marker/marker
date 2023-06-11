use crate::{
    ast::{impl_callable_data_trait, stmt::StmtKind, BodyId, CommonCallableData, Ident},
    ffi::{FfiOption, FfiSlice},
};

use super::{CommonExprData, ExprKind};

/// A block expression is one of the most fundamental expressions in Rust. It
/// is used by items and expressions to group statements together and express
/// scopes.
///
/// ```
/// //       vv The function body has an empty block
/// fn foo() {}
///
/// //      vvvvvvv An unsafe block
/// let _ = unsafe {
///     1 + 2
/// //  ^^^^^ An expression which value is returned from the block, indicated
/// //        by the missing semicolon at the end.
/// };
///
/// //      vvvvvv An optional label to be targeted by break expressions
/// let _ = 'label: {
///     12
/// };
/// ```
///
/// [`BlockExpr`] nodes are often simply called *blocks*, while the optional
/// expression at the end of a block is called *block expression*. The meaning
/// depends on the context. Marker's documentation will try to make the meaning
/// clear by linking directly to the [`BlockExpr`] struct or calling it a *block*.
#[repr(C)]
#[derive(Debug)]
pub struct BlockExpr<'ast> {
    data: CommonExprData<'ast>,
    stmts: FfiSlice<'ast, StmtKind<'ast>>,
    expr: FfiOption<ExprKind<'ast>>,
    label: FfiOption<Ident<'ast>>,
    is_unsafe: bool,
}

impl<'ast> BlockExpr<'ast> {
    /// This returns all statements of this block. The optional value expression,
    /// which is returned by the block, is stored separately. See [`BlockExpr::expr()`]
    pub fn stmts(&self) -> &[StmtKind<'ast>] {
        self.stmts.get()
    }

    /// Blocks may optionally end with an expression, indicated by an expression
    /// without a trailing semicolon.
    pub fn expr(&self) -> Option<ExprKind<'ast>> {
        self.expr.copy()
    }

    pub fn label(&self) -> Option<&Ident<'ast>> {
        self.label.get()
    }

    pub fn is_unsafe(&self) -> bool {
        self.is_unsafe
    }
}

super::impl_expr_data!(BlockExpr<'ast>, Block);

#[cfg(feature = "driver-api")]
impl<'ast> BlockExpr<'ast> {
    pub fn new(
        data: CommonExprData<'ast>,
        stmts: &'ast [StmtKind<'ast>],
        expr: Option<ExprKind<'ast>>,
        label: Option<Ident<'ast>>,
        is_unsafe: bool,
    ) -> Self {
        Self {
            data,
            stmts: stmts.into(),
            expr: expr.into(),
            label: label.into(),
            is_unsafe,
        }
    }
}

/// A closure expression
///
/// ```
/// //          vvvvvvvvvvvvvvvvvvvvvvvvvvvvv A Closure expression
/// let print = |name| println!("Hey {name}");
/// //           ^^^^  ^^^^^^^^^^^^^^^^^^^^^ The body of the closure
/// //           |
/// //           A named argument
///
/// //          vvvv The `move` keyword specifying the capture kind of the closure
/// let msger = move || {
///     print("Marker")
/// };
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct ClosureExpr<'ast> {
    data: CommonExprData<'ast>,
    callable_data: CommonCallableData<'ast>,
    capture_kind: CaptureKind,
    body: BodyId,
}

impl<'ast> ClosureExpr<'ast> {
    pub fn capture_kind(&self) -> CaptureKind {
        self.capture_kind
    }

    pub fn body(&self) -> BodyId {
        self.body
    }
}

super::impl_expr_data!(ClosureExpr<'ast>, Closure);

impl_callable_data_trait!(ClosureExpr<'ast>);

#[cfg(feature = "driver-api")]
impl<'ast> ClosureExpr<'ast> {
    pub fn new(
        data: CommonExprData<'ast>,
        callable_data: CommonCallableData<'ast>,
        capture_kind: CaptureKind,
        body: BodyId,
    ) -> Self {
        Self {
            data,
            callable_data,
            capture_kind,
            body,
        }
    }
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum CaptureKind {
    Default,
    Move,
}
