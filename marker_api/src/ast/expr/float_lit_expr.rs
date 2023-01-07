use crate::ffi::FfiOption;

use super::{CommonExprData, ExprPrecedence};

/// A float literal like `1.0`, `2e-2`, `2_f32`. The results of float operations
/// can be hardware-dependent. For exact value checks, it might be better to check
/// the written float literal by getting the code snipped from the expression span.
/// See:
/// * [`ExprData::span()`](`super::ExprData::span`)
/// * [`Span::snippet()`](`crate::ast::Span::snippet`)
///
/// All integer literals are unsigned, negative numbers have a unary negation
/// operation as their parent.
#[repr(C)]
#[derive(Debug)]
pub struct FloatLitExpr<'ast> {
    data: CommonExprData<'ast>,
    value: f64,
    suffix: FfiOption<FloatSuffix>,
}

impl<'ast> FloatLitExpr<'ast> {
    /// The semantic value of the written literal. The results of float operations
    /// can be hardware-dependent. For exact value checks, it might be better to check
    /// the written float literal from the span snippet or check for a range around the
    /// value in question.
    ///
    /// All integer literals are unsigned, negative numbers have a unary negation
    /// operation as their parent.
    pub fn value(&self) -> f64 {
        self.value
    }

    /// The suffix if it has been defined by the user. Use the
    /// [`ExprData::ty`](`super::ExprData::ty`) method to determine the type,
    /// if it hasn't been specified in the suffix
    pub fn suffix(&self) -> Option<FloatSuffix> {
        self.suffix.copy()
    }
}

super::impl_expr_data!(
    FloatLitExpr<'ast>,
    FloatLit,
    fn precedence(&self) -> ExprPrecedence {
        ExprPrecedence::Lit
    }
);

#[cfg(feature = "driver-api")]
impl<'ast> FloatLitExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, value: f64, suffix: Option<FloatSuffix>) -> Self {
        Self {
            data,
            value,
            suffix: suffix.into(),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FloatSuffix {
    F32,
    F64,
}
