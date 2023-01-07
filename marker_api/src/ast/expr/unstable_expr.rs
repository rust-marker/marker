use super::{CommonExprData, ExprPrecedence};

#[repr(C)]
#[derive(Debug)]
pub struct UnstableExpr<'ast> {
    data: CommonExprData<'ast>,
    /// For this expression, we need to specifically store the precedence, as
    /// this could represent different expressions with different precedence.
    precedence: ExprPrecedence,
}

super::impl_expr_data!(
    UnstableExpr<'ast>,
    Unstable,
    fn precedence(&self) -> ExprPrecedence {
        self.precedence
    }
);

#[cfg(feature = "driver-api")]
impl<'ast> UnstableExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, precedence: ExprPrecedence) -> Self {
        Self { data, precedence }
    }
}
