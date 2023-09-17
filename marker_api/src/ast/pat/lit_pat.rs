use crate::ast::expr::LitExprKind;

use super::CommonPatData;

/// A literal expression inside a pattern.
///
/// ```
/// # let string = "marker remix";
/// match string {
///     "example" => true,
/// //  ^^^^^^^^^ A string literal used as a pattern
///     _ => false,
/// };
/// ```
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct LitPat<'ast> {
    data: CommonPatData<'ast>,
    lit: LitExprKind<'ast>,
}

impl<'ast> LitPat<'ast> {
    /// The literal expression used as a pattern.
    pub fn lit(&self) -> LitExprKind<'ast> {
        self.lit
    }
}

super::impl_pat_data!(LitPat<'ast>, Lit);
