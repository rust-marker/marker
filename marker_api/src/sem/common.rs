use std::fmt::Debug;
use std::marker::PhantomData;

/// The semantic equivalent of a [`ConstExpr`][crate::ast::expr::ConstExpr], at
/// least theoretically. This part of the API is sadly not done yet, so this is
/// just a placeholder.
///
/// See: rust-marker/marker#179
#[repr(C)]
pub struct ConstValue<'ast> {
    _lifetime: PhantomData<&'ast ()>,
}

impl<'ast> Debug for ConstValue<'ast> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConstValue {{ /* WIP: See rust-marker/marker#179 */}}")
            .finish()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ConstValue<'ast> {
    pub fn new() -> Self {
        Self { _lifetime: PhantomData }
    }
}
