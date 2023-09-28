use crate::{ast::SymbolId, context::with_cx, ffi::FfiOption, ffi::FfiSlice};

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

/// A float literal like `1.0`, `2e-2`, `2_f32`. The results of float operations
/// can be hardware-dependent. For exact value checks, it might be better to check
/// the written float literal by getting the code snipped from the expression span.
/// See:
/// * [`HasSpan::span()`](`super::HasSpan::span`)
/// * [`Span::snippet()`](`crate::span::Span::snippet`)
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

#[repr(C)]
#[derive(Debug)]
pub struct StrLitExpr<'ast> {
    data: CommonExprData<'ast>,
    is_raw: bool,
    str_data: StrLitData<'ast>,
}

impl<'ast> StrLitExpr<'ast> {
    /// Returns `true`, if this is a raw string literal, like `r#"Hello World!"#`
    pub fn is_raw_lit(&self) -> bool {
        self.is_raw
    }

    /// Returns `true`, if this is a standard string literal, like `"Hello World!"`.
    /// This type of string is also sometimes referred to as *Cooked*.
    pub fn is_standard_lit(&self) -> bool {
        !self.is_raw
    }

    /// This returns `true`, if the literal is a byte string literal like `b"Hello\0"`
    /// or `br#"World"#`
    pub fn is_byte_str(&self) -> bool {
        matches!(self.str_data, StrLitData::Bytes(_))
    }

    /// This returns the UTF-8 string value of the string, if possible. Normal
    /// and raw strings in Rust are required to be UTF-8. Byte strings will be
    /// converted to UTF-8 if possible, otherwise `None` will be returned
    pub fn str_value(&self) -> Option<&str> {
        match &self.str_data {
            StrLitData::Sym(sym) => Some(with_cx(self, |cx| cx.symbol_str(*sym))),
            StrLitData::Bytes(bytes) => std::str::from_utf8(bytes.get()).ok(),
        }
    }

    /// Returns the value of the string as bytes.
    pub fn byte_value(&self) -> &[u8] {
        match &self.str_data {
            StrLitData::Sym(sym) => with_cx(self, |cx| cx.symbol_str(*sym)).as_bytes(),
            StrLitData::Bytes(bytes) => bytes.get(),
        }
    }
}

super::impl_expr_data!(
    StrLitExpr<'ast>,
    StrLit,
    fn precedence(&self) -> ExprPrecedence {
        ExprPrecedence::Lit
    }
);

#[cfg(feature = "driver-api")]
impl<'ast> StrLitExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, is_raw: bool, str_data: StrLitData<'ast>) -> Self {
        Self { data, is_raw, str_data }
    }
}

#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
enum StrLitData<'ast> {
    Sym(SymbolId),
    /// A byte string might not be valid UTF-8
    Bytes(FfiSlice<'ast, u8>),
}
