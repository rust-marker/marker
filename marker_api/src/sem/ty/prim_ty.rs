use crate::common::{NumKind, TextKind};

use super::CommonTyData;

/// The semantic representation of the [`bool`] type.
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct BoolTy<'ast> {
    data: CommonTyData<'ast>,
}

impl<'ast> std::fmt::Display for BoolTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("bool").finish()
    }
}

super::impl_ty_data!(BoolTy<'ast>, Bool);

/// The semantic representation of a numeric type like [`u32`], [`i32`], [`f64`].
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct NumTy<'ast> {
    data: CommonTyData<'ast>,
    numeric_kind: NumKind,
}

impl<'ast> NumTy<'ast> {
    pub fn numeric_kind(&self) -> NumKind {
        self.numeric_kind
    }

    pub fn is_signed(&self) -> bool {
        self.numeric_kind.is_signed()
    }

    pub fn is_unsigned(&self) -> bool {
        self.numeric_kind.is_unsigned()
    }

    pub fn is_float(&self) -> bool {
        self.numeric_kind.is_float()
    }

    pub fn is_integer(&self) -> bool {
        self.numeric_kind.is_integer()
    }
}

super::impl_ty_data!(NumTy<'ast>, Num);

impl<'ast> std::fmt::Display for NumTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.numeric_kind)
    }
}

/// The semantic representation of a textual type like [`char`] or [`str`].
#[repr(C)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct TextTy<'ast> {
    data: CommonTyData<'ast>,
    textual_kind: TextKind,
}

impl<'ast> TextTy<'ast> {
    pub fn textual_kind(&self) -> TextKind {
        self.textual_kind
    }

    pub fn is_str(&self) -> bool {
        matches!(self.textual_kind, TextKind::Str)
    }

    pub fn is_char(&self) -> bool {
        matches!(self.textual_kind, TextKind::Char)
    }
}

super::impl_ty_data!(TextTy<'ast>, Text);

impl<'ast> std::fmt::Debug for TextTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.textual_kind)
    }
}

/// The semantic representation of the never type [`!`](prim@never).
#[repr(C)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct NeverTy<'ast> {
    data: CommonTyData<'ast>,
}

super::impl_ty_data!(NeverTy<'ast>, Never);

impl<'ast> std::fmt::Debug for NeverTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("!").finish()
    }
}
