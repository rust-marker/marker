use std::mem::{size_of, transmute};

use marker_api::ast::{BodyId, CrateId, GenericId, ItemId, SpanId, SymbolId, TyDefId};

use rustc_hir as hir;

#[must_use]
pub fn to_crate_id(rustc_id: hir::def_id::CrateNum) -> CrateId {
    assert_eq!(size_of::<CrateId>(), 4);
    CrateId::new(rustc_id.as_u32())
}

#[must_use]
pub fn to_rustc_crate_num(api_id: CrateId) -> hir::def_id::CrateNum {
    assert_eq!(size_of::<CrateId>(), 4);
    hir::def_id::CrateNum::from_u32(api_id.data())
}

#[must_use]
pub fn to_rustc_item_id(api_id: ItemId) -> hir::ItemId {
    assert_eq!(size_of::<ItemId>(), size_of::<ItemIdLayout>(), "the layout is invalid");
    // # Safety
    // The layout is validated with the `assert` above
    let layout: ItemIdLayout = unsafe { transmute(api_id) };
    hir::ItemId {
        owner_id: hir::OwnerId {
            def_id: hir::def_id::LocalDefId {
                local_def_index: hir::def_id::DefIndex::from_u32(layout.index),
            },
        },
    }
}

#[repr(C)]
struct TyDefIdLayout {
    krate: u32,
    index: u32,
}

#[must_use]
pub fn to_ty_def_id(rustc_id: hir::def_id::DefId) -> TyDefId {
    assert_eq!(
        size_of::<TyDefId>(),
        size_of::<TyDefIdLayout>(),
        "the layout is invalid"
    );
    let layout = TyDefIdLayout {
        krate: rustc_id.krate.as_u32(),
        index: rustc_id.index.as_u32(),
    };
    // # Safety
    // The layout is validated with the `assert` above
    unsafe { transmute(layout) }
}

#[repr(C)]
struct GenericIdLayout {
    krate: u32,
    index: u32,
}

#[must_use]
pub fn to_generic_id(rustc_id: hir::def_id::DefId) -> GenericId {
    assert_eq!(
        size_of::<GenericId>(),
        size_of::<GenericIdLayout>(),
        "the layout is invalid"
    );
    let layout = GenericIdLayout {
        krate: rustc_id.krate.as_u32(),
        index: rustc_id.index.as_u32(),
    };
    // # Safety
    // The layout is validated with the `assert` above
    unsafe { transmute(layout) }
}

#[repr(C)]
struct BodyIdLayout {
    // Note: AFAIK rustc only loads bodies from the current crate, this allows
    // rustc to only store the index of the `DefId` and leave out the crate index.
    // Other drivers, will most likely require additional information, like the
    // crate id,
    owner: u32,
    index: u32,
}

#[must_use]
pub fn to_api_body_id(rustc_id: hir::BodyId) -> BodyId {
    assert_eq!(size_of::<BodyId>(), size_of::<BodyIdLayout>(), "the layout is invalid");
    let layout = BodyIdLayout {
        owner: rustc_id.hir_id.owner.def_id.local_def_index.as_u32(),
        index: rustc_id.hir_id.local_id.as_u32(),
    };
    // # Safety
    // The layout is validated with the `assert` above
    unsafe { transmute(layout) }
}

#[must_use]
pub fn to_rustc_body_id(api_id: BodyId) -> hir::BodyId {
    assert_eq!(size_of::<BodyId>(), size_of::<BodyIdLayout>(), "the layout is invalid");
    // # Safety
    // The layout is validated with the `assert` above
    let layout: BodyIdLayout = unsafe { transmute(api_id) };
    hir::BodyId {
        hir_id: hir::HirId {
            owner: hir::OwnerId {
                def_id: hir::def_id::LocalDefId {
                    local_def_index: hir::def_id::DefIndex::from_u32(layout.owner),
                },
            },
            local_id: hir::hir_id::ItemLocalId::from_u32(layout.index),
        },
    }
}

#[must_use]
pub fn to_span_id(rustc_span: rustc_span::Span) -> SpanId {
    assert_eq!(
        size_of::<SpanId>(),
        size_of::<rustc_span::Span>(),
        "the size of `Span` or `SpanId` has changed"
    );
    // # Safety
    // The site was validated with the `assert` above, the layout is provided by rustc
    unsafe { transmute(rustc_span) }
}

#[must_use]
pub fn to_rustc_span_from_id(api_id: SpanId) -> rustc_span::Span {
    assert_eq!(
        size_of::<SpanId>(),
        size_of::<rustc_span::Span>(),
        "the size of `Span` or `SpanId` has changed"
    );
    // # Safety
    // The site was validated with the `assert` above, the layout is provided by rustc
    unsafe { transmute(api_id) }
}

#[must_use]
pub fn to_symbol_id(sym: rustc_span::Symbol) -> SymbolId {
    assert_eq!(size_of::<SymbolId>(), 4);
    SymbolId::new(sym.as_u32())
}

#[must_use]
pub fn to_rustc_symbol(api_id: SymbolId) -> rustc_span::Symbol {
    assert_eq!(size_of::<SymbolId>(), 4);
    assert_eq!(size_of::<rustc_span::Symbol>(), 4);
    // FIXME: `rustc_span::Symbol` currently has no public constructor for the
    // index value and no `#[repr(C)]` attribute. Therefore, this conversion is
    // unsound. This requires changes in rustc.
    unsafe { transmute(api_id) }
}

// /////////////////////////////////////////////////////////
// API Item ID
// /////////////////////////////////////////////////////////

#[repr(C)]
pub struct ItemIdLayout {
    krate: u32,
    index: u32,
}

pub fn to_item_id(id: impl Into<ItemIdLayout>) -> ItemId {
    let layout: ItemIdLayout = id.into();
    assert_eq!(size_of::<ItemId>(), size_of::<ItemIdLayout>(), "the layout is invalid");
    // # Safety
    // The layout is validated with the `assert` above
    unsafe { transmute(layout) }
}

impl From<hir::ItemId> for ItemIdLayout {
    fn from(value: hir::ItemId) -> Self {
        // My understanding is, that the `owner_id` is the `DefId` of this item.
        // We'll see if this holds true, when marker crashes and burns ^^
        value.owner_id.def_id.into()
    }
}
impl From<hir::def_id::LocalDefId> for ItemIdLayout {
    fn from(value: hir::def_id::LocalDefId) -> Self {
        value.to_def_id().into()
    }
}
impl From<hir::OwnerId> for ItemIdLayout {
    fn from(value: hir::OwnerId) -> Self {
        value.to_def_id().into()
    }
}
impl From<hir::def_id::DefId> for ItemIdLayout {
    fn from(rustc_id: hir::def_id::DefId) -> Self {
        ItemIdLayout {
            krate: rustc_id.krate.as_u32(),
            index: rustc_id.index.as_u32(),
        }
    }
}

// /////////////////////////////////////////////////////////
// Rustc DefId
// /////////////////////////////////////////////////////////

pub trait ToDefIdInfo {
    fn into_crate_and_index(self) -> (u32, u32);
}

#[must_use]
pub fn to_rustc_def_id(api_id: impl ToDefIdInfo) -> hir::def_id::DefId {
    let (index, krate) = api_id.into_crate_and_index();
    hir::def_id::DefId {
        index: hir::def_id::DefIndex::from_u32(index),
        krate: hir::def_id::CrateNum::from_u32(krate),
    }
}

impl ToDefIdInfo for ItemId {
    fn into_crate_and_index(self) -> (u32, u32) {
        assert_eq!(size_of::<ItemId>(), size_of::<ItemIdLayout>(), "the layout is invalid");
        // # Safety
        // The layout is validated with the `assert` above
        let layout: ItemIdLayout = unsafe { transmute(self) };
        (layout.index, layout.krate)
    }
}

impl ToDefIdInfo for TyDefId {
    fn into_crate_and_index(self) -> (u32, u32) {
        assert_eq!(
            size_of::<TyDefId>(),
            size_of::<TyDefIdLayout>(),
            "the layout is invalid"
        );
        // # Safety
        // The layout is validated with the `assert` above
        let layout: TyDefIdLayout = unsafe { transmute(self) };
        (layout.index, layout.krate)
    }
}

impl ToDefIdInfo for GenericId {
    fn into_crate_and_index(self) -> (u32, u32) {
        assert_eq!(
            size_of::<GenericId>(),
            size_of::<GenericIdLayout>(),
            "the layout is invalid"
        );
        // # Safety
        // The layout is validated with the `assert` above
        let layout: GenericIdLayout = unsafe { transmute(self) };
        (layout.index, layout.krate)
    }
}
