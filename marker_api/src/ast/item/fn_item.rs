use crate::ast::generic::SynGenericParams;
use crate::ast::pat::PatKind;
use crate::ast::ty::SynTyKind;
use crate::ast::{Abi, BodyId, Constness, Safety, SpanId, Syncness};
use crate::context::with_cx;
use crate::ffi::{FfiOption, FfiSlice};
use crate::prelude::Span;

use super::CommonItemData;

/// A function item like:
///
/// ```
/// // A function with a parameter and a body
/// pub fn foo(bot: u32) {}
///
/// # pub struct SomeItem;
/// impl SomeItem {
///     // A function as an associated item, with a body
///     pub fn bar(&self) {}
/// }
///
/// pub trait SomeTrait {
///     // A function without a body
///     fn baz(_: i32);
/// }
/// ```
///
/// See: <https://doc.rust-lang.org/reference/items/functions.html>
#[repr(C)]
#[derive(Debug)]
pub struct FnItem<'ast> {
    data: CommonItemData<'ast>,
    generics: SynGenericParams<'ast>,
    constness: Constness,
    syncness: Syncness,
    safety: Safety,
    is_extern: bool,
    has_self: bool,
    abi: Abi,
    params: FfiSlice<'ast, FnParam<'ast>>,
    return_ty: FfiOption<SynTyKind<'ast>>,
    body_id: FfiOption<BodyId>,
}

super::impl_item_data!(FnItem, Fn);

impl<'ast> FnItem<'ast> {
    pub fn generics(&self) -> &SynGenericParams<'ast> {
        &self.generics
    }

    /// The [`BodyId`] of this function. It can be `None` in trait definitions
    /// or for extern functions.
    pub fn body_id(&self) -> Option<BodyId> {
        self.body_id.copy()
    }

    /// Returns the [`Constness`] of this callable
    pub fn constness(&self) -> Constness {
        self.constness
    }

    /// Returns the [`Syncness`] of this callable.
    ///
    /// Use this to check if the function is async.
    pub fn syncness(&self) -> Syncness {
        self.syncness
    }

    /// Returns the [`Safety`] of this callable.
    ///
    /// Use this to check if the function is unsafe.
    pub fn safety(&self) -> Safety {
        self.safety
    }

    /// Returns `true`, if this callable is marked as `extern`. Bare functions
    /// only use the `extern` keyword to specify the ABI. These will currently
    /// still return `false` even if the keyword is present. In those cases,
    /// please refer to the [`abi()`](`Self::abi`) instead.
    ///
    /// Defaults to `false` if unspecified.
    pub fn is_extern(&self) -> bool {
        self.is_extern
    }

    /// Returns the [`Abi`] of the callable.
    pub fn abi(&self) -> Abi {
        self.abi
    }

    /// Returns `true`, if this callable has a specified `self` argument. The
    /// type of `self` can be retrieved from the first element of
    /// [`params()`](`Self::params`).
    pub fn has_self(&self) -> bool {
        self.has_self
    }

    /// Returns the parameters, that this callable accepts. The `self` argument
    /// of methods, will be the first element of this slice. Use
    /// [`has_self()`](`Self::has_self`) to determine if the first argument is `self`.
    pub fn params(&self) -> &[FnParam<'ast>] {
        self.params.get()
    }

    /// The return type of this callable, if specified.
    pub fn return_ty(&self) -> Option<&SynTyKind<'ast>> {
        self.return_ty.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> FnItem<'ast> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        data: CommonItemData<'ast>,
        generics: SynGenericParams<'ast>,
        constness: Constness,
        syncness: Syncness,
        safety: Safety,
        is_extern: bool,
        has_self: bool,
        abi: Abi,
        params: &'ast [FnParam<'ast>],
        return_ty: Option<SynTyKind<'ast>>,
        body: Option<BodyId>,
    ) -> Self {
        Self {
            data,
            generics,
            constness,
            syncness,
            safety,
            is_extern,
            has_self,
            abi,
            params: params.into(),
            return_ty: return_ty.into(),
            body_id: body.into(),
        }
    }
}

/// A parameter for a [`FnItem`], like:
///
/// ```
/// // A parameter with a name and type
/// //                     vvvvvvvvv
/// fn function_with_ident(name: u32) {}
/// // A parameter with a pattern and type
/// //                       vvvvvvvvvvvvvvvvvv
/// fn function_with_pattern((a, b): (u32, i32)) {}
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct FnParam<'ast> {
    span: SpanId,
    pat: PatKind<'ast>,
    ty: SynTyKind<'ast>,
}

impl<'ast> FnParam<'ast> {
    pub fn span(&self) -> &Span<'ast> {
        with_cx(self, |cx| cx.span(self.span))
    }

    pub fn pat(&self) -> PatKind<'ast> {
        self.pat
    }

    pub fn ty(&self) -> SynTyKind<'ast> {
        self.ty
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> FnParam<'ast> {
    pub fn new(span: SpanId, pat: PatKind<'ast>, ty: SynTyKind<'ast>) -> Self {
        Self { span, pat, ty }
    }
}
