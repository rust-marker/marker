use std::fmt::Debug;
use std::marker::PhantomData;

use crate::{
    common::{HasNodeId, SpanId, StmtId},
    ffi::FfiOption,
    private::Sealed,
    span::{HasSpan, Span},
};

use super::{expr::ExprKind, item::ItemKind, pat::PatKind, ty::TyKind};

/// This trait combines methods, which all statements have in common.
///
/// This trait is only meant to be implemented inside this crate. The `Sealed`
/// super trait prevents external implementations.
pub trait StmtData<'ast>: Debug + HasSpan<'ast> + HasNodeId + Sealed {
    /// Returns the [`StmtId`] of this statement
    fn id(&self) -> StmtId;
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone)]
pub enum StmtKind<'ast> {
    Item(&'ast ItemStmt<'ast>),
    Let(&'ast LetStmt<'ast>),
    Expr(&'ast ExprStmt<'ast>),
}

impl<'ast> StmtKind<'ast> {
    pub fn id(&self) -> StmtId {
        match self {
            StmtKind::Item(node, ..) => node.id(),
            StmtKind::Let(node, ..) => node.id(),
            StmtKind::Expr(node, ..) => node.id(),
        }
    }

    pub fn span(&self) -> &Span<'ast> {
        match self {
            StmtKind::Item(node, ..) => node.span(),
            StmtKind::Let(node, ..) => node.span(),
            StmtKind::Expr(node, ..) => node.span(),
        }
    }

    /// Returns the attributes attached to this statement.
    ///
    /// Currently, it's only a placeholder until a proper representation is implemented.
    /// rust-marker/marker#51 tracks the task of implementing this. You're welcome to
    /// leave any comments in that issue.
    pub fn attrs(&self) {}
}

crate::span::impl_spanned_for!(StmtKind<'ast>);
crate::common::impl_identifiable_for!(StmtKind<'ast>);
impl<'ast> crate::private::Sealed for StmtKind<'ast> {}

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
struct CommonStmtData<'ast> {
    /// The lifetime is not needed right now, but it's safer to include it for
    /// future additions. Having it in this struct makes it easier for all
    /// pattern structs, as they will have a valid use for `'ast` even if they
    /// don't need it. Otherwise, we might need to declare this field in each
    /// pattern.
    #[cfg_attr(feature = "driver-api", builder(default))]
    _lifetime: PhantomData<&'ast ()>,
    id: StmtId,
    span: SpanId,
}

macro_rules! impl_stmt_data {
    ($self_ty:ty, $enum_name:ident) => {
        impl<'ast> StmtData<'ast> for $self_ty {
            fn id(&self) -> crate::common::StmtId {
                self.data.id
            }
        }

        impl<'ast> $crate::span::HasSpan<'ast> for $self_ty {
            fn span(&self) -> &crate::span::Span<'ast> {
                $crate::context::with_cx(self, |cx| cx.span(self.data.span))
            }
        }

        $crate::common::impl_identifiable_for!($self_ty, use StmtData);

        impl<'ast> From<&'ast $self_ty> for $crate::ast::stmt::StmtKind<'ast> {
            fn from(from: &'ast $self_ty) -> Self {
                $crate::ast::stmt::StmtKind::$enum_name(from)
            }
        }

        impl<'ast> $crate::private::Sealed for $self_ty {}
    };
}

use impl_stmt_data;

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct LetStmt<'ast> {
    data: CommonStmtData<'ast>,
    pat: PatKind<'ast>,
    #[cfg_attr(feature = "driver-api", builder(setter(into)))]
    ty: FfiOption<TyKind<'ast>>,
    #[cfg_attr(feature = "driver-api", builder(setter(into)))]
    init: FfiOption<ExprKind<'ast>>,
    #[cfg_attr(feature = "driver-api", builder(setter(into)))]
    els: FfiOption<ExprKind<'ast>>,
}

impl<'ast> LetStmt<'ast> {
    pub fn pat(&self) -> PatKind<'ast> {
        self.pat
    }

    /// Returns the syntactic type, if it has been specified.
    pub fn ty(&self) -> Option<TyKind<'ast>> {
        self.ty.copy()
    }

    pub fn init(&self) -> Option<ExprKind<'ast>> {
        self.init.copy()
    }

    /// This returns the optional `else` expression of the let statement.
    ///
    /// `els` is an abbreviation for `else`, which is a reserved keyword in Rust.
    pub fn els(&self) -> Option<ExprKind> {
        self.els.copy()
    }
}

impl_stmt_data!(LetStmt<'ast>, Let);

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct ExprStmt<'ast> {
    data: CommonStmtData<'ast>,
    expr: ExprKind<'ast>,
}

impl<'ast> ExprStmt<'ast> {
    pub fn expr(&self) -> ExprKind<'ast> {
        self.expr
    }
}

impl_stmt_data!(ExprStmt<'ast>, Expr);

#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct ItemStmt<'ast> {
    data: CommonStmtData<'ast>,
    item: ItemKind<'ast>,
}

impl<'ast> ItemStmt<'ast> {
    pub fn item(&self) -> ItemKind<'ast> {
        self.item
    }
}

impl_stmt_data!(ItemStmt<'ast>, Item);
