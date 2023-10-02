use crate::common::{NumKind, TextKind};

use super::CommonSynTyData;

/// The syntactic representation of the [`bool`] type.
#[repr(C)]
#[derive(Debug)]
pub struct SynBoolTy<'ast> {
    data: CommonSynTyData<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> SynBoolTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>) -> Self {
        Self { data }
    }
}

super::impl_ty_data!(SynBoolTy<'ast>, Bool);

impl<'ast> std::fmt::Display for SynBoolTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("bool").finish()
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct SynNumTy<'ast> {
    data: CommonSynTyData<'ast>,
    numeric_kind: NumKind,
}

impl<'ast> SynNumTy<'ast> {
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

super::impl_ty_data!(SynNumTy<'ast>, Num);

#[cfg(feature = "driver-api")]
impl<'ast> SynNumTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, numeric_kind: NumKind) -> Self {
        Self { data, numeric_kind }
    }
}

impl<'ast> std::fmt::Display for SynNumTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.numeric_kind)
    }
}

/// The syntactic representation of a textual type like [`char`] or [`str`].
#[repr(C)]
#[derive(Debug)]
pub struct SynTextTy<'ast> {
    data: CommonSynTyData<'ast>,
    textual_kind: TextKind,
}

super::impl_ty_data!(SynTextTy<'ast>, Text);

impl<'ast> SynTextTy<'ast> {
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

impl<'ast> std::fmt::Display for SynTextTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.textual_kind)
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SynTextTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>, textual_kind: TextKind) -> Self {
        Self { data, textual_kind }
    }
}

/// The syntactic representation of the never type [`!`](prim@never).
#[repr(C)]
pub struct SynNeverTy<'ast> {
    data: CommonSynTyData<'ast>,
}

super::impl_ty_data!(SynNeverTy<'ast>, Never);

impl<'ast> std::fmt::Debug for SynNeverTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("!").finish()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SynNeverTy<'ast> {
    pub fn new(data: CommonSynTyData<'ast>) -> Self {
        Self { data }
    }
}
