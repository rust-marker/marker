mod id;
pub use id::*;
mod span;
pub use span::*;

use std::fmt::Debug;

#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Edition {
    Edition2015,
    Edition2018,
    Edition2021,
}

#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Applicability {
    /// The suggestion is definitely what the user intended, or maintains the exact meaning of the
    /// code. This suggestion should be automatically applied.
    ///
    /// In case of multiple `MachineApplicable` suggestions (whether as part of
    /// the same `multipart_suggestion` or not), all of them should be
    /// automatically applied.
    MachineApplicable,

    /// The suggestion may be what the user intended, but it is uncertain. The suggestion should
    /// result in valid Rust code if it is applied.
    MaybeIncorrect,

    /// The suggestion contains placeholders like `(...)` or `{ /* fields */ }`. The suggestion
    /// cannot be applied automatically because it will not result in valid Rust code. The user
    /// will need to fill in the placeholders.
    HasPlaceholders,

    /// The suggestion can not be automatically applied or the applicability is unknown.
    Unspecified,
}

/// Used to indicate the safety. [`Safety::Default`] is the default safe rust mode.
#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Safety {
    Default,
    Unsafe,
}

#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Constness {
    Default,
    Const,
}

#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Asyncness {
    Default,
    Async,
}

#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Abi {
    /// This is the default of the current driver, the actual ABI can vary between
    /// implementations. In general this means that the user has not selected a
    /// specific ABI.
    Default,
    /// FIXME: Remove this variant. See
    /// <https://doc.rust-lang.org/nightly/nightly-rustc/rustc_target/spec/abi/enum.Abi.html>
    Other,
}

pub struct Spanned<'ast, T> {
    pub node: T,
    pub span: &'ast Span<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast, T> Spanned<'ast, T> {
    #[must_use]
    pub fn new(node: T, span: &'ast Span<'ast>) -> Self {
        Self { node, span }
    }
}

#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Symbol {
    index: u32,
}

#[cfg(feature = "driver-api")]
impl Symbol {
    #[must_use]
    pub fn new(index: u32) -> Self {
        Self { index }
    }
}

pub type Ident<'ast> = Spanned<'ast, Symbol>;

pub trait Attribute<'ast>: Debug {
    // FIXME: Add attribute functions
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ItemPath<'ast> {
    segments: &'ast [PathSegment],
    target: PathResolution,
}

impl<'ast> ItemPath<'ast> {
    pub fn get_segments(&self) -> &[PathSegment] {
        self.segments
    }

    pub fn resolve(&self) -> &PathResolution {
        &self.target
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> ItemPath<'ast> {
    pub fn new(segments: &'ast [PathSegment], target: PathResolution) -> Self {
        Self { segments, target }
    }
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct PathSegment {
    /// This symbol can correspond to an empty string in some cases for example
    /// for [turbo fish paths](https://turbo.fish/::%3Cchecker%3E).
    name: Symbol,
    /// The item or object, that this segment resolves to.
    target: PathResolution,
    // FIXME: Represent more complext paths like:
    // ```rs
    // <S as Type>::item
    // Vec::<u8>::with_capacity(1024)
    // Iterator<Item = u32>
    // ```
}

#[cfg(feature = "driver-api")]
impl PathSegment {
    #[must_use]
    pub fn new(name: Symbol, target: PathResolution) -> Self {
        Self { name, target }
    }
}

impl PathSegment {
    pub fn get_name(&self) -> Symbol {
        self.name
    }

    pub fn get_target(&self) -> &PathResolution {
        &self.target
    }
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum PathResolution {
    Item(ItemId),
    /// An path belonging to a tool. This will for instance be used for attributes
    /// like:
    /// ```ignore
    /// #[clippy::msrv]
    /// #[rustfmt::skip]
    /// ```
    ToolItem,
    /// The path could not be resolved.
    Unresolved,
}

pub trait Lifetime<'ast>: Debug {
    // FIXME: Add functions for lifetimes, see <https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/sty/struct.Region.html>
}

pub trait Pattern<'ast> {}
