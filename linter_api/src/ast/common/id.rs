/// This ID uniquely identifies a crate during linting, the id is not stable
/// between different sessions.
///
/// The layout and size of this type might change. The id will continue to
/// provide the current trait implementations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CrateId {
    /// The layout of the data is up to the driver implementation. The API will never
    /// create custom IDs and pass them to the driver. The size of this type might
    /// change. Drivers should validate the size with tests.
    data: u32,
}

#[cfg(feature = "driver-api")]
impl CrateId {
    #[must_use]
    pub fn new(data: u32) -> Self {
        Self { data }
    }

    pub fn data(self) -> u32 {
        self.data
    }
}

/// This ID uniquely identifies an item during linting, the id is not stable
/// between different sessions.
///
/// The layout and size of this type might change. The id will continue to
/// provide the current trait implementations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ItemId {
    /// The layout of the data is up to the driver implementation. The API will never
    /// create custom IDs and pass them to the driver. The size of this type might
    /// change. Drivers should validate the size with tests.
    data: u64,
}

#[cfg(feature = "driver-api")]
impl ItemId {
    pub fn new(data: u64) -> Self {
        Self { data }
    }

    pub fn data(&self) -> u64 {
        self.data
    }
}

/// This ID uniquely identifies a body during linting, the id is not stable
/// between different sessions.
///
/// The layout and size of this type might change. The id will continue to
/// provide the current trait implementations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BodyId {
    /// The layout of the data is up to the driver implementation. The API will never
    /// create custom IDs and pass them to the driver. The size of this type might
    /// change. Drivers should validate the size with tests.
    data: u64,
}

#[cfg(feature = "driver-api")]
impl BodyId {
    #[must_use]
    pub fn new(data: u64) -> Self {
        Self { data }
    }

    pub fn data(self) -> u64 {
        self.data
    }
}

/// **Unstable**
///
/// This id is used to identify [`Span`]s. This type is only intended for internal
/// use. Lint crates should always get a [`Span`] object.
#[repr(C)]
#[doc(hidden)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
pub(crate) struct SpanId {
    /// The layout of the data is up to the driver implementation. The API will never
    /// create custom IDs and pass them to the driver. The size of this type might
    /// change. Drivers should validate the size with tests.
    data: u64,
}

#[cfg(feature = "driver-api")]
impl SpanId {
    #[must_use]
    pub fn new(data: u64) -> Self {
        Self { data }
    }

    pub fn data(self) -> u64 {
        self.data
    }
}
