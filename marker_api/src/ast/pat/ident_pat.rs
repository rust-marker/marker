use crate::{
    ast::{Mutability, SymbolId, VarId},
    context::with_cx,
    ffi::FfiOption,
};

use super::{CommonPatData, PatKind};

#[repr(C)]
#[derive(Debug)]
pub struct IdentPat<'ast> {
    data: CommonPatData<'ast>,
    name: SymbolId,
    var: VarId,
    mutability: Mutability,
    is_ref: bool,
    binding_pat: FfiOption<PatKind<'ast>>,
}

impl<'ast> IdentPat<'ast> {
    pub fn name(&self) -> &str {
        with_cx(self, |cx| cx.symbol_str(self.name))
    }

    pub fn var_id(&self) -> VarId {
        self.var
    }

    pub fn mutability(&self) -> Mutability {
        self.mutability
    }

    pub fn is_ref(&self) -> bool {
        self.is_ref
    }

    /// The pattern, if the variable originates from a binding to a pattern.
    /// ```
    /// # let expr = 10;
    /// match expr {
    ///     x @ 0.. => println!("{x} is positive"),
    /// //  ^   ^^^
    /// //  |     +-- The pattern, that the variable is bound to
    /// //  |
    /// //  +-------- The bound variable
    ///     _ => println!("x is most likely negative"),
    /// }
    /// ```
    pub fn binding_pat(&self) -> Option<PatKind<'ast>> {
        self.binding_pat.copy()
    }
}

super::impl_pat_data!(IdentPat<'ast>, Ident);

#[cfg(feature = "driver-api")]
impl<'ast> IdentPat<'ast> {
    pub fn new(
        data: CommonPatData<'ast>,
        name: SymbolId,
        var: VarId,
        mutability: Mutability,
        is_ref: bool,
        binding_pat: Option<PatKind<'ast>>,
    ) -> Self {
        Self {
            data,
            name,
            var,
            mutability,
            is_ref,
            binding_pat: binding_pat.into(),
        }
    }
}
