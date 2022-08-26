/// This ID uniquely identifies a crate during compilation.
///
/// The ID of a specific crate can change between different compilations.
#[repr(C)]
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

/// This ID uniquely identifies a body during compilation.
///
/// The ID of a specific body can change between different compilations.
#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

/// **Unstable**
///
/// This id is used to identify [`Span`]s. This type is only intended for internal
/// use. Lint crates should always get a [`Span`] object.
///
/// The layout of the data is up to the driver implementation. The API will never
/// create custom IDs and pass them to the driver. The size of this type might
/// change. Drivers should validate the size with tests.
#[repr(C)]
#[doc(hidden)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
pub(crate) struct SpanId {
    data: u64,
}

#[cfg(feature = "driver-api")]
impl SpanId {
    #[must_use]
    pub fn new(data: u64) -> Self {
        Self { data }
    }

    pub fn get_data(self) -> u64 {
        self.data
    }
}
