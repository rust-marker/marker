use std::fmt::Debug;
use std::marker::PhantomData;

mod args;
pub use args::*;
mod param;
pub use param::*;

use crate::{
    common::{GenericId, SpanId, SymbolId},
    context::with_cx,
    ffi::{FfiOption, FfiSlice},
    span::Span,
};

/// A lifetime used as a generic argument or on a reference like this:
///
/// ```
/// # use core::marker::PhantomData;
/// # #[derive(Default)]
/// # struct Item<'a> {
/// #     _data: PhantomData<&'a ()>,
/// # }
///
/// # fn example<'a>() {
/// let _item: Item<'_> = Item::default();
/// //              ^^
/// let _item: Item<'a> = Item::default();
/// //              ^^
/// let _item: &'static str = "Hello world";
/// //          ^^^^^^^
/// # }
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct Lifetime<'ast> {
    _lifetime: PhantomData<&'ast ()>,
    span: FfiOption<SpanId>,
    kind: LifetimeKind,
}

#[repr(C)]
#[derive(Debug)]
#[allow(clippy::exhaustive_enums)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
pub(crate) enum LifetimeKind {
    /// A lifetime with a label like `'ast`
    Label(SymbolId, GenericId),
    /// The magic `'static` lifetime
    Static,
    /// The mysterious `'_` lifetime
    Infer,
}

impl<'ast> Lifetime<'ast> {
    /// This returns the [`GenericId`] of this lifetime, if it's labeled, or [`None`]
    /// otherwise. `'static` will also return [`None`]
    pub fn id(&self) -> Option<GenericId> {
        match self.kind {
            LifetimeKind::Label(_, id) => Some(id),
            _ => None,
        }
    }

    /// Note that the `'static` lifetime is not a label and will therefore return [`None`]
    pub fn label(&self) -> Option<&str> {
        match self.kind {
            LifetimeKind::Label(sym, _) => Some(with_cx(self, |cx| cx.symbol_str(sym))),
            _ => None,
        }
    }

    pub fn has_label(&self) -> bool {
        matches!(self.kind, LifetimeKind::Label(..))
    }

    pub fn is_static(&self) -> bool {
        matches!(self.kind, LifetimeKind::Static)
    }

    pub fn is_infer(&self) -> bool {
        matches!(self.kind, LifetimeKind::Infer)
    }

    pub fn span(&self) -> Option<&Span<'ast>> {
        self.span.get().map(|span| with_cx(self, |cx| cx.span(*span)))
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> Lifetime<'ast> {
    pub fn new(span: Option<SpanId>, kind: LifetimeKind) -> Self {
        Self {
            _lifetime: PhantomData,
            span: span.into(),
            kind,
        }
    }
}

/// The syntactic representation of generic arguments for an item or path.
///
/// ```
/// # use std::fmt::Debug;
/// //             vv This is a generic argument
/// generic_item::<u8>(32);
///
/// pub fn generic_item<T: Copy>(t: T)
/// //                  ^^^^^^^ This is a generic parameter
/// where
///     T: Debug,
/// //  ^^^^^^^^ This is a bound for a generic parameter
/// {
///     println!("{:#?}", t);
/// }
/// ```
///
/// See:
/// * [`SynGenericParams`]
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(Clone))]
pub struct SynGenericArgs<'ast> {
    args: FfiSlice<'ast, SynGenericArgKind<'ast>>,
}

impl<'ast> SynGenericArgs<'ast> {
    pub fn args(&self) -> &'ast [SynGenericArgKind<'ast>] {
        self.args.get()
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SynGenericArgs<'ast> {
    pub fn new(args: &'ast [SynGenericArgKind<'ast>]) -> Self {
        Self { args: args.into() }
    }
}

/// A singular generic argument.
///
/// See: <https://doc.rust-lang.org/stable/reference/paths.html>
#[repr(C)]
#[non_exhaustive]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(Clone))]
pub enum SynGenericArgKind<'ast> {
    /// A lifetime as a generic argument, like this:
    ///
    /// ```
    /// # use std::marker::PhantomData;
    /// # #[derive(Default)]
    /// # pub struct HasLifetime<'a> {
    /// #     _data: PhantomData<&'a ()>,
    /// # }
    /// let _foo: HasLifetime<'static> = HasLifetime::default();
    /// //                    ^^^^^^^
    /// ```
    Lifetime(&'ast SynLifetimeArg<'ast>),
    /// A type as a generic argument, like this:
    ///
    /// ```
    /// let _bar: Vec<String> = vec![];
    /// //            ^^^^^^
    /// ```
    Ty(&'ast SynTyArg<'ast>),
    /// A type binding as a generic argument, like this:
    ///
    /// ```ignore
    /// let _baz: &dyn Iterator<Item=String> = todo!();
    /// //                      ^^^^^^^^^^^
    /// ```
    Binding(&'ast SynBindingArg<'ast>),
    /// A constant expression as a generic argument, like this:
    ///
    /// ```ignore
    /// # struct Vec<const N: usize> {
    /// #     data: [f32; N],
    /// # }
    /// #
    /// let _bat: Vec<3> = todo!();
    /// //            ^
    /// ```
    Const(&'ast SynConstArg<'ast>),
}

/// This represents the generic parameters of a generic item. The bounds applied
/// to the parameters in the declaration are stored as clauses in this struct.
///
/// ```
/// # use std::fmt::Debug;
/// pub fn generic_item<T: Copy>(t: T)
/// //                  ^^^^^^^ This is a generic parameter
/// where
///     T: Debug,
/// //  ^^^^^^^^  This is a bound for a generic parameter
/// {
///     println!("{:#?}", t);
/// }
///
/// //             vv This is a generic argument
/// generic_item::<u8>(32);
/// ```
/// See
/// * [`SynGenericArgs`]
#[repr(C)]
#[derive(Debug)]
pub struct SynGenericParams<'ast> {
    params: FfiSlice<'ast, SynGenericParamKind<'ast>>,
    clauses: FfiSlice<'ast, SynWhereClauseKind<'ast>>,
}

impl<'ast> SynGenericParams<'ast> {
    pub fn params(&self) -> &'ast [SynGenericParamKind<'ast>] {
        self.params.get()
    }

    pub fn clauses(&self) -> &'ast [SynWhereClauseKind<'ast>] {
        self.clauses.get()
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> SynGenericParams<'ast> {
    pub fn new(params: &'ast [SynGenericParamKind<'ast>], clauses: &'ast [SynWhereClauseKind<'ast>]) -> Self {
        Self {
            params: params.into(),
            clauses: clauses.into(),
        }
    }
}
