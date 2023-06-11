use std::marker::PhantomData;

use super::{NumKind, TextKind};

#[repr(C)]
pub struct SemBoolTy<'ast> {
    _lt: PhantomData<&'ast ()>,
}

#[cfg(feature = "driver-api")]
impl<'ast> SemBoolTy<'ast> {
    pub fn new() -> Self {
        Self { _lt: PhantomData }
    }
}

impl<'ast> std::fmt::Debug for SemBoolTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("bool").finish()
    }
}

#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
pub struct SemNumTy<'ast> {
    _ast: PhantomData<&'ast ()>,
    numeric_kind: NumKind,
}

#[cfg(feature = "driver-api")]
impl<'ast> SemNumTy<'ast> {
    pub fn new(numeric_kind: NumKind) -> Self {
        Self {
            _ast: PhantomData,
            numeric_kind,
        }
    }
}

impl<'ast> SemNumTy<'ast> {
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

impl<'ast> std::fmt::Debug for SemNumTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.numeric_kind)
    }
}

#[repr(C)]
pub struct SemTextTy<'ast> {
    _ast: PhantomData<&'ast ()>,
    textual_kind: TextKind,
}

#[cfg(feature = "driver-api")]
impl<'ast> SemTextTy<'ast> {
    pub fn new(textual_kind: TextKind) -> Self {
        Self {
            _ast: PhantomData,
            textual_kind,
        }
    }
}

impl<'ast> SemTextTy<'ast> {
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

impl<'ast> std::fmt::Debug for SemTextTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.textual_kind)
    }
}

#[repr(C)]
pub struct SemNeverTy<'ast> {
    _lt: PhantomData<&'ast ()>,
}

#[cfg(feature = "driver-api")]
impl<'ast> SemNeverTy<'ast> {
    pub fn new() -> Self {
        Self { _lt: PhantomData }
    }
}

impl<'ast> std::fmt::Debug for SemNeverTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("!").finish()
    }
}
