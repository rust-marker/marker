/// This ID uniquely identifies a crate during compilation.
///
/// The ID of a specific crate can change between different compilations.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
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

/// This ID uniquely identifies a body during compilation.
///
/// The ID of a specific body can change between different compilations.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(C)]
pub struct BodyId {
    owner: usize,
    index: usize,
}

#[cfg(feature = "driver-api")]
impl BodyId {
    #[must_use]
    pub fn new(owner: usize, index: usize) -> Self {
        Self { owner, index }
    }

    pub fn get_data(self) -> (usize, usize) {
        (self.owner, self.index)
    }
}
