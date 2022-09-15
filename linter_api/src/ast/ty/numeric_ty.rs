use super::CommonTyData;

#[repr(C)]
pub struct NumericTy<'ast> {
    data: CommonTyData<'ast>,
    numeric_kind: NumericKind,
}

#[cfg(feature = "driver-api")]
impl<'ast> NumericTy<'ast> {
    pub fn new(data: CommonTyData<'ast>, numeric_kind: NumericKind) -> Self {
        Self { data, numeric_kind }
    }
}

super::impl_ty_data!(NumericTy<'ast>, Numeric);

impl<'ast> NumericTy<'ast> {
    // FIXME: Do we want to keep this method and expose the enum or hide
    // it completly behind methods?
    pub fn numeric_kind(&self) -> NumericKind {
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

impl<'ast> std::fmt::Debug for NumericTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.numeric_kind)
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NumericKind {
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

impl NumericKind {
    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            NumericKind::Isize
                | NumericKind::I8
                | NumericKind::I16
                | NumericKind::I32
                | NumericKind::I64
                | NumericKind::I128
                | NumericKind::F32
                | NumericKind::F64
        )
    }

    pub fn is_unsigned(&self) -> bool {
        !self.is_signed()
    }

    pub fn is_float(&self) -> bool {
        matches!(self, NumericKind::F32 | NumericKind::F64)
    }

    pub fn is_integer(&self) -> bool {
        !self.is_float()
    }
}

impl std::fmt::Debug for NumericKind {
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
