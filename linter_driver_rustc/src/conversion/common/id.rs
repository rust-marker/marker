use std::mem::{size_of, transmute};

use linter_api::ast::{BodyId, CrateId, GenericId, ItemId, SpanId, SymbolId, TyDefId};

use crate::context::RustcContext;

pub fn to_api_crate_id(_cx: &RustcContext<'_, '_>, rustc_id: rustc_hir::def_id::CrateNum) -> CrateId {
    assert_eq!(size_of::<CrateId>(), 4);
    CrateId::new(rustc_id.as_u32())
}

pub fn to_rustc_krate_id(_cx: &RustcContext<'_, '_>, api_id: CrateId) -> rustc_hir::def_id::CrateNum {
    assert_eq!(size_of::<CrateId>(), 4);
    rustc_hir::def_id::CrateNum::from_u32(api_id.data())
}

#[repr(C)]
struct ItemIdLayout {
    krate: u32,
    index: u32,
}

pub fn to_api_item_id(cx: &RustcContext<'_, '_>, rustc_id: rustc_hir::ItemId) -> ItemId {
    to_api_item_id_from_def_id(cx, rustc_id.owner_id.to_def_id())
}

pub fn to_api_item_id_from_def_id(_cx: &RustcContext<'_, '_>, rustc_id: rustc_hir::def_id::DefId) -> ItemId {
    assert_eq!(size_of::<ItemId>(), size_of::<ItemIdLayout>(), "the layout is invalid");
    let layout = ItemIdLayout {
        krate: rustc_id.krate.as_u32(),
        index: rustc_id.index.as_u32(),
    };
    // # Safety
    // The layout is validated with the `assert` above
    unsafe { transmute(layout) }
}

pub fn to_rustc_def_id_from_item_id(_cx: &RustcContext<'_, '_>, api_id: ItemId) -> rustc_hir::def_id::DefId {
    assert_eq!(size_of::<ItemId>(), size_of::<ItemIdLayout>(), "the layout is invalid");
    // # Safety
    // The layout is validated with the `assert` above
    let layout: ItemIdLayout = unsafe { transmute(api_id) };
    rustc_hir::def_id::DefId {
        index: rustc_hir::def_id::DefIndex::from_u32(layout.index),
        krate: rustc_hir::def_id::CrateNum::from_u32(layout.krate),
    }
}

pub fn to_rustc_item_id(_cx: &RustcContext<'_, '_>, api_id: ItemId) -> rustc_hir::ItemId {
    assert_eq!(size_of::<ItemId>(), size_of::<ItemIdLayout>(), "the layout is invalid");
    // # Safety
    // The layout is validated with the `assert` above
    let layout: ItemIdLayout = unsafe { transmute(api_id) };
    rustc_hir::ItemId {
        owner_id: rustc_hir::OwnerId {
            def_id: rustc_hir::def_id::LocalDefId {
                local_def_index: rustc_hir::def_id::DefIndex::from_u32(layout.index),
            },
        },
    }
}

#[repr(C)]
struct TyDefIdLayout {
    krate: u32,
    index: u32,
}

pub fn to_api_ty_def_id(_cx: &RustcContext<'_, '_>, rustc_id: rustc_hir::def_id::DefId) -> TyDefId {
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

pub fn to_rustc_def_id_from_ty_def_id(_cx: &RustcContext<'_, '_>, api_id: TyDefId) -> rustc_hir::def_id::DefId {
    assert_eq!(
        size_of::<TyDefId>(),
        size_of::<TyDefIdLayout>(),
        "the layout is invalid"
    );
    // # Safety
    // The layout is validated with the `assert` above
    let layout: TyDefIdLayout = unsafe { transmute(api_id) };
    rustc_hir::def_id::DefId {
        index: rustc_hir::def_id::DefIndex::from_u32(layout.index),
        krate: rustc_hir::def_id::CrateNum::from_u32(layout.krate),
    }
}

#[repr(C)]
struct GenericIdLayout {
    krate: u32,
    index: u32,
}

pub fn to_api_generic_id(_cx: &RustcContext<'_, '_>, rustc_id: rustc_hir::def_id::DefId) -> GenericId {
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

pub fn to_rustc_def_id_from_generic_id(_cx: &RustcContext<'_, '_>, api_id: GenericId) -> rustc_hir::def_id::DefId {
    assert_eq!(
        size_of::<GenericId>(),
        size_of::<GenericIdLayout>(),
        "the layout is invalid"
    );
    // # Safety
    // The layout is validated with the `assert` above
    let layout: GenericIdLayout = unsafe { transmute(api_id) };
    rustc_hir::def_id::DefId {
        index: rustc_hir::def_id::DefIndex::from_u32(layout.index),
        krate: rustc_hir::def_id::CrateNum::from_u32(layout.krate),
    }
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

pub fn to_api_body_id(_cx: &RustcContext<'_, '_>, rustc_id: rustc_hir::BodyId) -> BodyId {
    assert_eq!(size_of::<BodyId>(), size_of::<BodyIdLayout>(), "the layout is invalid");
    let layout = BodyIdLayout {
        owner: rustc_id.hir_id.owner.def_id.local_def_index.as_u32(),
        index: rustc_id.hir_id.local_id.as_u32(),
    };
    // # Safety
    // The layout is validated with the `assert` above
    unsafe { transmute(layout) }
}

pub fn to_rustc_body_id(_cx: &RustcContext<'_, '_>, api_id: BodyId) -> rustc_hir::BodyId {
    assert_eq!(size_of::<BodyId>(), size_of::<BodyIdLayout>(), "the layout is invalid");
    // # Safety
    // The layout is validated with the `assert` above
    let layout: BodyIdLayout = unsafe { transmute(api_id) };
    rustc_hir::BodyId {
        hir_id: rustc_hir::HirId {
            owner: rustc_hir::OwnerId {
                def_id: rustc_hir::def_id::LocalDefId {
                    local_def_index: rustc_hir::def_id::DefIndex::from_u32(layout.owner),
                },
            },
            local_id: rustc_hir::hir_id::ItemLocalId::from_u32(layout.index),
        },
    }
}

pub fn to_api_span_id(_cx: &RustcContext<'_, '_>, rustc_span: rustc_span::Span) -> SpanId {
    assert_eq!(
        size_of::<SpanId>(),
        size_of::<rustc_span::Span>(),
        "the size of `Span` or `SpanId` has changed"
    );
    // # Safety
    // The site was validated with the `assert` above, the layout is provided by rustc
    unsafe { transmute(rustc_span) }
}

pub fn to_rustc_span_from_id(_cx: &RustcContext<'_, '_>, api_id: SpanId) -> rustc_span::Span {
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
pub fn to_api_symbol_id(sym: rustc_span::Symbol) -> SymbolId {
    assert_eq!(size_of::<SymbolId>(), 4);
    SymbolId::new(sym.as_u32())
}

pub fn to_rustc_symbol(_cx: &RustcContext<'_, '_>, api_id: SymbolId) -> rustc_span::Symbol {
    assert_eq!(size_of::<SymbolId>(), 4);
    assert_eq!(size_of::<rustc_span::Symbol>(), 4);
    // FIXME: `rustc_span::Symbol` currently has no public constructor for the
    // index value and no `#[repr(C)]` attribute. Therefore, this conversion is
    // unsound. This requires changes in rustc.
    unsafe { transmute(api_id) }
}
