use super::{CommonExprData, ExprPrecedence};

#[repr(C)]
#[derive(Debug)]
pub struct CharLitExpr<'ast> {
    data: CommonExprData<'ast>,
    value: char,
}

impl<'ast> CharLitExpr<'ast> {
    pub fn value(&self) -> char {
        self.value
    }
}

super::impl_expr_data!(
    CharLitExpr<'ast>,
    CharLit,
    fn precedence(&self) -> ExprPrecedence {
        ExprPrecedence::Lit
    }
);

#[cfg(feature = "driver-api")]
impl<'ast> CharLitExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, value: char) -> Self {
        Self { data, value }
    }
}
