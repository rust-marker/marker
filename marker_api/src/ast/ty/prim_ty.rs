use crate::common::{NumKind, TextKind};

use super::CommonSynTyData;

/// The syntactic representation of the [`bool`] type.
#[repr(C)]
#[derive(Debug)]
pub struct BoolTy<'ast> {
    data: CommonSynTyData<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> BoolTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>) -> Self {
        Self { data }
    }
}

super::impl_ty_data!(BoolTy<'ast>, Bool);

impl<'ast> std::fmt::Display for BoolTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("bool").finish()
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct NumTy<'ast> {
    data: CommonSynTyData<'ast>,
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

#[cfg(feature = "driver-api")]
impl<'ast> NumTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, numeric_kind: NumKind) -> Self {
        Self { data, numeric_kind }
    }
}

impl<'ast> std::fmt::Display for NumTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.numeric_kind)
    }
}

/// The syntactic representation of a textual type like [`char`] or [`str`].
#[repr(C)]
#[derive(Debug)]
pub struct TextTy<'ast> {
    data: CommonSynTyData<'ast>,
    textual_kind: TextKind,
}

super::impl_ty_data!(TextTy<'ast>, Text);

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

impl<'ast> std::fmt::Display for TextTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.textual_kind)
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> TextTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, textual_kind: TextKind) -> Self {
        Self { data, textual_kind }
    }
}

/// The syntactic representation of the never type [`!`](prim@never).
#[repr(C)]
pub struct NeverTy<'ast> {
    data: CommonSynTyData<'ast>,
}

super::impl_ty_data!(NeverTy<'ast>, Never);

impl<'ast> std::fmt::Debug for NeverTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("!").finish()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> NeverTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>) -> Self {
        Self { data }
    }
}
