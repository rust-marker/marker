use crate::ffi::FfiOption;

use super::{item::ItemKind, ty::TyKind, pat::PatKind};

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum StmtKind<'ast> {
    Item(&'ast ItemKind<'ast>),
    Let(&'ast LetStmt<'ast>),
    // FIXME: Add expression variant
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct LetStmt<'ast> {
    pat: PatKind<'ast>,
    ty: FfiOption<TyKind<'ast>>,
    // TODO add optional init expression
    // TODO add optional else expression
}

impl<'ast> LetStmt<'ast> {
    pub fn pat(&self) -> PatKind<'ast> {
        self.pat
    }

    /// Returns the syntactic type, if it has been specified.
    pub fn ty(&self) -> Option<TyKind<'ast>> {
        self.ty.copy()
    }

    // FIXME: Add new method once expressions kind of exist.
}

