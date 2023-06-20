use std::marker::PhantomData;

use super::CommonSynTyData;

/// The syntactic representation of the [`bool`] type.
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
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

/// The semantic representation of the [`bool`] type.
#[repr(C)]
#[derive(Debug)]
pub struct SemBoolTy<'ast> {
    _lt: PhantomData<&'ast ()>,
}

#[cfg(feature = "driver-api")]
impl<'ast> SemBoolTy<'ast> {
    pub fn new() -> Self {
        Self { _lt: PhantomData }
    }
}

impl<'ast> std::fmt::Display for SemBoolTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("bool").finish()
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
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

/// The semantic representation of a numeric type like [`u32`], [`i32`], [`f64`].
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
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

impl<'ast> std::fmt::Display for SemNumTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.numeric_kind)
    }
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumKind {
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
    F32,
    F64,
}

impl NumKind {
    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            NumKind::Isize
                | NumKind::I8
                | NumKind::I16
                | NumKind::I32
                | NumKind::I64
                | NumKind::I128
                | NumKind::F32
                | NumKind::F64
        )
    }

    pub fn is_unsigned(&self) -> bool {
        !self.is_signed()
    }

    pub fn is_float(&self) -> bool {
        matches!(self, NumKind::F32 | NumKind::F64)
    }

    pub fn is_integer(&self) -> bool {
        !self.is_float()
    }
}

impl std::fmt::Display for NumKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Isize => write!(f, "isize"),
            Self::I8 => write!(f, "i8"),
            Self::I16 => write!(f, "i16"),
            Self::I32 => write!(f, "i32"),
            Self::I64 => write!(f, "i64"),
            Self::I128 => write!(f, "i128"),
            Self::Usize => write!(f, "usize"),
            Self::U8 => write!(f, "u8"),
            Self::U16 => write!(f, "u16"),
            Self::U32 => write!(f, "u32"),
            Self::U64 => write!(f, "u64"),
            Self::U128 => write!(f, "u128"),
            Self::F32 => write!(f, "f32"),
            Self::F64 => write!(f, "f64"),
        }
    }
}

/// The syntactic representation of a textual type like [`char`] or [`str`].
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
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

/// The semantic representation of a textual type like [`char`] or [`str`].
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
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextKind {
    Char,
    Str,
}

impl std::fmt::Display for TextKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Char => write!(f, "char"),
            Self::Str => write!(f, "str"),
        }
    }
}

/// The syntactic representation of the never type [`!`](prim@never).
#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
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

/// The semantic representation of the never type [`!`](prim@never).
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
