use super::CommonTyData;

#[repr(C)]
#[derive(PartialEq, Eq, Hash)]
pub struct TextTy<'ast> {
    data: CommonTyData<'ast>,
    textual_kind: TextKind,
}

#[cfg(feature = "driver-api")]
impl<'ast> TextTy<'ast> {
    pub fn new(data: CommonTyData<'ast>, textual_kind: TextKind) -> Self {
        Self { data, textual_kind }
    }
}

super::impl_ty_data!(TextTy<'ast>, Text);

impl<'ast> TextTy<'ast> {
    // FIXME: Do we want to keep this method and expose the enum or hide
    // it completly behind methods?
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

impl<'ast> std::fmt::Debug for TextTy<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.textual_kind)
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextKind {
    Char,
    Str,
}

impl std::fmt::Debug for TextKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Char => write!(f, "char"),
            Self::Str => write!(f, "str"),
        }
    }
}
