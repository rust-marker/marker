use crate::{ast::SymbolId, context::with_cx, ffi::FfiSlice};

use super::{CommonExprData, ExprPrecedence};

#[repr(C)]
#[derive(Debug)]
pub struct StrLitExpr<'ast> {
    data: CommonExprData<'ast>,
    str_data: StrKindWithData<'ast>,
}

impl<'ast> StrLitExpr<'ast> {
    pub fn str_kind(&self) -> StrKind {
        match &self.str_data {
            StrKindWithData::Str(_) => StrKind::Str,
            StrKindWithData::Raw(_) => StrKind::Raw,
            StrKindWithData::Byte(_) => StrKind::Byte,
        }
    }

    /// This returns the UTF-8 string value of the string, if possible. Normal
    /// and raw strings in Rust are required to be UTF-8. Byte strings will be
    /// converted to UTF-8 if possible, otherwise `None` will be returned
    pub fn str_value(&self) -> Option<&str> {
        match &self.str_data {
            StrKindWithData::Str(sym) | StrKindWithData::Raw(sym) => Some(with_cx(self, |cx| cx.symbol_str(*sym))),
            StrKindWithData::Byte(bytes) => std::str::from_utf8(bytes.get()).ok(),
        }
    }

    /// Returns the value of the string as bytes.
    pub fn byte_value(&self) -> &[u8] {
        match &self.str_data {
            StrKindWithData::Str(sym) | StrKindWithData::Raw(sym) => with_cx(self, |cx| cx.symbol_str(*sym)).as_bytes(),
            StrKindWithData::Byte(bytes) => bytes.get(),
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

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StrKind {
    /// A normal standard string like `"Hello world!"`
    Str,
    /// A raw string like `r#"Hello World!"#`
    Raw,
    /// A byte string like `b"Hello world!\0"`. The content of a byte string doesn't
    /// have to be valid UTF-8
    Byte,
}

#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
enum StrKindWithData<'ast> {
    Str(SymbolId),
    Raw(SymbolId),
    /// A byte string doesn't have to be valid UTF-8
    Byte(FfiSlice<'ast, u8>),
}
