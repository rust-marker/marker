use crate::{
    ast::ty::TyKind,
    ffi::{FfiOption, FfiSlice},
};

use super::{GenericParams, Lifetime, TyParamBound};

/// This represents a single clause in a where statement
///
/// ```
/// fn foo<'a, T>()
/// where
///     'a: 'static,
///     T: Iterator + 'a,
///     T::Item: Copy,
///     String: PartialEq<T>,
///     i32: Default,
/// {}
/// ```
#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum WhereClauseKind<'ast> {
    Lifetime(&'ast LifetimeClause<'ast>),
    Ty(&'ast TyClause<'ast>),
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct LifetimeClause<'ast> {
    lifetime: Lifetime<'ast>,
    bounds: FfiSlice<'ast, Lifetime<'ast>>,
}

impl<'ast> LifetimeClause<'ast> {
    pub fn lifetime(&self) -> &Lifetime<'ast> {
        &self.lifetime
    }

    pub fn bounds(&self) -> &[Lifetime<'ast>] {
        self.bounds.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> LifetimeClause<'ast> {
    pub fn new(lifetime: Lifetime<'ast>, bounds: &'ast [Lifetime<'ast>]) -> Self {
        Self {
            lifetime,
            bounds: bounds.into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct TyClause<'ast> {
    params: FfiOption<GenericParams<'ast>>,
    ty: TyKind<'ast>,
    bounds: FfiSlice<'ast, TyParamBound<'ast>>,
}

impl<'ast> TyClause<'ast> {
    /// Additional parameters introduced as a part of this where clause with a `for`.
    pub fn params(&self) -> Option<&GenericParams<'ast>> {
        self.params.get()
    }

    /// The bound type
    pub fn ty(&self) -> TyKind<'ast> {
        self.ty
    }

    /// The bounds applied to the specified type.
    pub fn bounds(&self) -> &'ast [TyParamBound<'ast>] {
        self.bounds.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> TyClause<'ast> {
    pub fn new(params: Option<GenericParams<'ast>>, ty: TyKind<'ast>, bounds: &'ast [TyParamBound<'ast>]) -> Self {
        Self {
            params: params.into(),
            ty,
            bounds: bounds.into(),
        }
    }
}
