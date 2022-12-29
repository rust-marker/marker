use super::{CommonExprData, ExprPrecedence};

#[repr(C)]
#[derive(Debug)]
pub struct BoolLitExpr<'ast> {
    data: CommonExprData<'ast>,
    value: bool,
}

impl<'ast> BoolLitExpr<'ast> {
    pub fn value(&self) -> bool {
        self.value
    }
}

super::impl_expr_data!(
    BoolLitExpr<'ast>,
    BoolLit,
    fn precedence(&self) -> ExprPrecedence {
        ExprPrecedence::Lit
    }
);

#[cfg(feature = "driver-api")]
impl<'ast> BoolLitExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, value: bool) -> Self {
        Self { data, value }
    }
}
