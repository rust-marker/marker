use std::fmt::Debug;

use super::item::ItemId;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrateId {
    index: u32,
}

#[cfg(feature = "driver-api")]
impl CrateId {
    #[must_use]
    pub fn new(index: u32) -> Self {
        Self { index }
    }

    pub fn get_data(self) -> u32 {
        self.index
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BodyId {
    krate: CrateId,
    index: u32,
}

#[cfg(feature = "driver-api")]
impl BodyId {
    #[must_use]
    pub fn new(krate: CrateId, index: u32) -> Self {
        Self { krate, index }
    }

    pub fn get_data(self) -> (CrateId, u32) {
        (self.krate, self.index)
    }
}

/// A `Span` represents a span of source code. It can be part of the source code
/// or part of generated logic using macros. Spans are used to determine the origin
/// of elements and to create suggestions and lint messages.
///
/// Note: When working with [`Span`]s and modifying their bounds it can happen that
/// they end inside a unicode character as they often safe the unicode position. These
/// cases can cause panics if they are used to access the underlying source code. The
/// normal provided [`Span`]s should all be fine.
pub trait Span<'ast>: Debug {
    fn is_from_expansion(&self) -> bool;

    fn in_derive_expansion(&self) -> bool;

    // Returns `true` if `self` fully encloses `other`
    fn contains(&self, other: &dyn Span<'ast>) -> bool;

    // Returns `true` if `self` touches `other`
    fn overlaps(&self, other: &dyn Span<'ast>) -> bool;

    // Edition of the crate from which this span came.
    fn edition(&self) -> Edition;

    /// Returns a `Span` that would enclose both `self` and `end`.
    ///
    /// ```text
    ///     ____             ___
    ///     self lorem ipsum end
    ///     ^^^^^^^^^^^^^^^^^^^^
    /// ```
    fn to(&'ast self, end: &dyn Span<'ast>) -> &dyn Span<'ast>;

    /// Returns a `Span` between the end of `self` to the beginning of `end`.
    ///
    /// ```text
    ///     ____             ___
    ///     self lorem ipsum end
    ///         ^^^^^^^^^^^^^
    /// ```
    fn between(&'ast self, end: &dyn Span<'ast>) -> &dyn Span<'ast>;

    /// Returns a `Span` from the beginning of `self` until the beginning of `end`.
    ///
    /// ```text
    ///     ____             ___
    ///     self lorem ipsum end
    ///     ^^^^^^^^^^^^^^^^^
    /// ```
    fn until(&'ast self, end: &dyn Span<'ast>) -> &dyn Span<'ast>;

    /// Returns the code that this span references or `None` if the code in unavailable
    fn snippet(&self) -> Option<String>;

    /// Converts a span to a code snippet if available, otherwise returns the default.
    ///
    /// This is useful if you want to provide suggestions for your lint or more generally, if you
    /// want to convert a given `Span` to a `String`. To create suggestions consider using
    /// [`Span::snippet_with_applicability`] to ensure that the [`Applicability`] stays correct.
    ///
    /// # Example
    /// ```rust,ignore
    /// // Given two spans one for `value` and one for the `init` expression.
    /// let value = Vec::new();
    /// //  ^^^^^   ^^^^^^^^^^
    /// //  span1   span2
    ///
    /// // The snipped call would return the corresponding code snippets
    /// span1.snippet_or_else("..") // -> "value"
    /// span2.snippet_or_else("..") // -> "Vec::new()"
    /// ```
    fn snippet_or_else(&self, default: &str) -> String {
        self.snippet().unwrap_or_else(|| default.to_string())
    }

    /// Same as [`Span::snippet`], but it adapts the applicability level by following rules:
    ///
    /// - Applicability level `Unspecified` will never be changed.
    /// - If the span is inside a macro, change the applicability level to `MaybeIncorrect`.
    /// - If the default value is used and the applicability level is `MachineApplicable`, change it
    ///   to
    /// `HasPlaceholders`
    fn snippet_with_applicability(&self, default: &str, applicability: &mut Applicability) -> String {
        if *applicability != Applicability::Unspecified && self.is_from_expansion() {
            *applicability = Applicability::MaybeIncorrect;
        }
        self.snippet().unwrap_or_else(|| {
            if *applicability == Applicability::MachineApplicable {
                *applicability = Applicability::HasPlaceholders;
            }
            default.to_string()
        })
    }

    /// Returns information about the File that this `Span` originates from if available.
    ///
    /// The structure is: `(<file>, <lint>, <column>)`
    ///
    /// FIXME: We should probably create a `File` struct or something for this (xFrednet)
    fn get_source_file(&self) -> Option<(String, u32, u32)>;
}

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

pub struct Spanned<'ast, T> {
    pub node: T,
    pub span: &'ast dyn Span<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast, T> Spanned<'ast, T> {
    #[must_use]
    pub fn new(node: T, span: &'ast dyn Span<'ast>) -> Self {
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

pub trait SimplePath<'ast>: Debug {
    fn get_segments(&'ast self) -> &'ast [Symbol];

    /// FIXME: There might be a better return type for this function. This also
    /// has to work for attributes like `#![clippy::msrv = "1.0.1"]`
    fn resolve(&'ast self) -> Option<ItemId>;
}

pub trait Lifetime<'ast>: Debug {
    // FIXME: Add functions for lifetimes, see <https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/sty/struct.Region.html>
}
