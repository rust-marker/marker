use crate::{
    ast::{pat::PatKind, stmt::StmtKind, ty::SynTyKind},
    common::{BodyId, Safety, SpanId, Syncness},
    context::with_cx,
    ffi::{FfiOption, FfiSlice},
    span::{Ident, Span},
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
///     let _ = 18;
/// //  ^^^^^^^^^^^
/// // A statement in the block
///     12
/// };
/// ```
///
/// [`BlockExpr`] nodes are often simply called *blocks*, while the optional
/// expression at the end of a block is called *block expression*. The meaning
/// depends on the context. Marker's documentation will try to make the meaning
/// clear by linking directly to the [`BlockExpr`] struct or calling it a *block*.
///
/// This expression also represents async blocks, the internal desugar used by
/// rustc is resugared for this.
#[repr(C)]
#[derive(Debug)]
pub struct BlockExpr<'ast> {
    data: CommonExprData<'ast>,
    stmts: FfiSlice<'ast, StmtKind<'ast>>,
    expr: FfiOption<ExprKind<'ast>>,
    label: FfiOption<Ident<'ast>>,
    safety: Safety,
    syncness: Syncness,
    capture_kind: CaptureKind,
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

    pub fn safety(&self) -> Safety {
        self.safety
    }

    pub fn syncness(&self) -> Syncness {
        self.syncness
    }

    /// The capture kind of this block. For normal blocks, this will always be
    /// [`CaptureKind::Default`], which in this context means no capture at all.
    /// Async blocks are special, as they can capture values by move, indicated
    /// by the `move` keyword, like in this:
    ///
    /// ```
    /// # use std::future::Future;
    /// # fn foo<'a>(x: &'a u8) -> impl Future<Output = u8> + 'a {
    /// // The whole block expression
    /// //  vvvvvvvvvvvvvvvvv
    ///     async move { *x }
    /// //        ^^^^
    /// // The move keyword defining the capture kind
    /// # }
    /// ```
    pub fn capture_kind(&self) -> CaptureKind {
        self.capture_kind
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
        safety: Safety,
        syncness: Syncness,
        capture_kind: CaptureKind,
    ) -> Self {
        Self {
            data,
            stmts: stmts.into(),
            expr: expr.into(),
            label: label.into(),
            safety,
            syncness,
            capture_kind,
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
    capture_kind: CaptureKind,
    params: FfiSlice<'ast, ClosureParam<'ast>>,
    return_ty: FfiOption<SynTyKind<'ast>>,
    body_id: BodyId,
}

impl<'ast> ClosureExpr<'ast> {
    pub fn capture_kind(&self) -> CaptureKind {
        self.capture_kind
    }

    pub fn params(&self) -> &'ast [ClosureParam<'ast>] {
        self.params.get()
    }

    pub fn return_ty(&self) -> Option<SynTyKind<'_>> {
        self.return_ty.copy()
    }

    pub fn body_id(&self) -> BodyId {
        self.body_id
    }
}

super::impl_expr_data!(ClosureExpr<'ast>, Closure);

#[cfg(feature = "driver-api")]
impl<'ast> ClosureExpr<'ast> {
    pub fn new(
        data: CommonExprData<'ast>,
        capture_kind: CaptureKind,
        params: &'ast [ClosureParam<'ast>],
        return_ty: Option<SynTyKind<'ast>>,
        body_id: BodyId,
    ) -> Self {
        Self {
            data,
            capture_kind,
            params: params.into(),
            return_ty: return_ty.into(),
            body_id,
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

/// A parameter for a [`ClosureExpr`], with a pattern and an optional type, like:
///
/// ```
/// # let _: fn(u32) -> () =
/// // A simple parameter
/// //   v
///     |x| { /*...*/ };
///
/// // A parameter with a type
/// //   vvvvvv
///     |y: u32| { /*...*/ };
///
/// # let _: fn((u32, u32, u32)) -> () =
/// // A parameter with a complex pattern
/// //   vvvvvvvv
///     |(a, b, c)| { /*...*/ };
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct ClosureParam<'ast> {
    span: SpanId,
    pat: PatKind<'ast>,
    ty: FfiOption<SynTyKind<'ast>>,
}

impl<'ast> ClosureParam<'ast> {
    pub fn span(&self) -> &Span<'ast> {
        with_cx(self, |cx| cx.span(self.span))
    }

    pub fn pat(&self) -> PatKind<'ast> {
        self.pat
    }

    pub fn ty(&self) -> Option<SynTyKind<'ast>> {
        self.ty.copy()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ClosureParam<'ast> {
    pub fn new(span: SpanId, pat: PatKind<'ast>, ty: Option<SynTyKind<'ast>>) -> Self {
        Self {
            span,
            pat,
            ty: ty.into(),
        }
    }
}
