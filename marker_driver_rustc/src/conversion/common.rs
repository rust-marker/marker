#[repr(C)]
pub struct GenericIdLayout {
    pub krate: u32,
    pub index: u32,
}

#[repr(C)]
pub struct TyDefIdLayout {
    pub krate: u32,
    pub index: u32,
}

#[repr(C)]
pub struct ItemIdLayout {
    pub krate: u32,
    pub index: u32,
}

#[repr(C)]
pub struct BodyIdLayout {
    // Note: AFAIK rustc only loads bodies from the current crate, this allows
    // rustc to only store the index of the `DefId` and leave out the crate index.
    // Other drivers, will most likely require additional information, like the
    // crate id,
    pub owner: u32,
    pub index: u32,
}

/// Used as a target for [`Into`] implementations, not that it shouldn't be
/// used as a transmute target. Instead the specific ID layouts should be used.
pub struct DefIdInfo {
    pub index: u32,
    pub krate: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct SpanSourceInfo {
    pub rustc_span_cx: rustc_span::hygiene::SyntaxContext,
    pub rustc_start_offset: usize,
}

#[repr(C)]
pub struct VarIdLayout {
    pub owner: u32,
    pub index: u32,
}
