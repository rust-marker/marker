use crate::ffi::FfiSlice;

mod args;
pub use args::*;
mod param;
pub use param::*;

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
