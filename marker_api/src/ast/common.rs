mod id;
pub use id::*;
mod ast_path;
pub use ast_path::*;

use std::fmt::Debug;

use super::generic::SynGenericArgs;

#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Edition {
    Edition2015,
    Edition2018,
    Edition2021,
}

#[non_exhaustive]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Abi {
    /// This is the default of the current driver, the actual ABI can vary between
    /// implementations. In general this means that the user has not selected a
    /// specific ABI.
    Default,
    C,
    /// FIXME: Remove this variant. See
    /// <https://doc.rust-lang.org/nightly/nightly-rustc/rustc_target/spec/abi/enum.Abi.html>
    Other,
}

#[repr(C)]
#[derive(Debug)]
pub struct TraitRef<'ast> {
    item_id: ItemId,
    generics: SynGenericArgs<'ast>,
}

#[cfg(feature = "driver-api")]
impl<'ast> TraitRef<'ast> {
    pub fn new(item_id: ItemId, generics: SynGenericArgs<'ast>) -> Self {
        Self { item_id, generics }
    }
}

impl<'ast> TraitRef<'ast> {
    pub fn trait_id(&self) -> ItemId {
        self.item_id
    }

    pub fn generics(&self) -> &SynGenericArgs<'ast> {
        &self.generics
    }
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Mutability {
    /// The object is mutable
    Mut,
    /// The object is unmutable
    Unmut,
}

impl Mutability {
    #[must_use]
    pub fn is_mut(&self) -> bool {
        matches!(self, Self::Mut)
    }
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Safety {
    Safe,
    Unsafe,
}

impl Safety {
    #[must_use]
    pub fn is_unsafe(&self) -> bool {
        matches!(self, Self::Unsafe)
    }
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Constness {
    Const,
    NotConst,
}

impl Constness {
    #[must_use]
    pub fn is_const(&self) -> bool {
        matches!(self, Self::Const)
    }
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Syncness {
    Sync,
    Async,
}

impl Syncness {
    #[must_use]
    pub fn is_sync(&self) -> bool {
        matches!(self, Self::Sync)
    }

    #[must_use]
    pub fn is_async(&self) -> bool {
        matches!(self, Self::Async)
    }
}
