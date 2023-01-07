use crate::ffi::FfiOption;

use super::{CommonExprData, ExprPrecedence};

/// A integer literal like `16` or `8_u8`. All integer literals in Rust are unsigned
/// numbers < 2^128. Negative numbers have a unary negation operation as their parent.
///
/// Values are casts into their respective type, the literal `300_u8` will have a value
/// 300 in the value field, but have the semantic value of `300 as u8` which is `44`.
#[repr(C)]
#[derive(Debug)]
pub struct IntLitExpr<'ast> {
    data: CommonExprData<'ast>,
    value: u128,
    suffix: FfiOption<IntSuffix>,
}

impl<'ast> IntLitExpr<'ast> {
    /// The value as a `u128`. Higher int literals are not allowed by rusts syntax.
    /// Negative numbers have a unary negation operation as their parent.
    pub fn value(&self) -> u128 {
        self.value
    }

    /// The suffix if it has been defined by the user. Use the
    /// [`ExprData::ty`](`super::ExprData::ty`) method to determine the type,
    /// if it hasn't been specified in the suffix
    pub fn suffix(&self) -> Option<IntSuffix> {
        self.suffix.copy()
    }
}

super::impl_expr_data!(
    IntLitExpr<'ast>,
    IntLit,
    fn precedence(&self) -> ExprPrecedence {
        ExprPrecedence::Lit
    }
);

#[cfg(feature = "driver-api")]
impl<'ast> IntLitExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, value: u128, suffix: Option<IntSuffix>) -> Self {
        Self {
            data,
            value,
            suffix: suffix.into(),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntSuffix {
    Isize,
    I8,
    I16,
    I32,
    I64,
    I128,
    Usize,
    U8,
    U16,
    U32,
    U64,
    U128,
}
