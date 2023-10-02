//! This module provides types, which are used by the semantic and syntactic
//! representations in Marker.

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
