use crate::{ast::SymbolId, context::with_cx, ffi::FfiSlice};

use super::{CommonExprData, ExprPrecedence};

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
