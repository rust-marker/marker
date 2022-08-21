use std::mem::{size_of, transmute};

use linter_api::ast::{BodyId, CrateId, ItemId};

use crate::context::RustcContext;

pub fn api_krate_id_from_rustc(_cx: &RustcContext<'_, '_>, rustc_id: rustc_hir::def_id::CrateNum) -> CrateId {
    assert_eq!(size_of::<CrateId>(), 4);
    CrateId::new(rustc_id.as_u32())
}

pub fn rustc_krate_id_from_api(_cx: &RustcContext<'_, '_>, api_id: CrateId) -> rustc_hir::def_id::CrateNum {
    assert_eq!(size_of::<CrateId>(), 4);
    rustc_hir::def_id::CrateNum::from_u32(api_id.data())
}

#[repr(C)]
struct ItemIdLayout {
    krate: u32,
    index: u32,
}

pub fn to_api_item_id(_cx: &RustcContext<'_, '_>, rustc_id: rustc_hir::def_id::DefId) -> ItemId {
    assert_eq!(size_of::<ItemId>(), size_of::<ItemIdLayout>(), "the layout is invalid");
    let layout = ItemIdLayout {
        krate: rustc_id.krate.as_u32(),
        index: rustc_id.index.as_u32(),
    };
    // # Safety
    // The layout is validated with the `assert` above
    unsafe { transmute(layout) }
}

pub fn to_rustc_def_id(_cx: &RustcContext<'_, '_>, api_id: ItemId) -> rustc_hir::def_id::DefId {
    assert_eq!(size_of::<ItemId>(), size_of::<ItemIdLayout>(), "the layout is invalid");
    // # Safety
    // The layout is validated with the `assert` above
    let layout: ItemIdLayout = unsafe { transmute(api_id) };
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
        owner: rustc_id.hir_id.owner.local_def_index.as_u32(),
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
            owner: rustc_hir::def_id::LocalDefId {
                local_def_index: rustc_hir::def_id::DefIndex::from_u32(layout.owner),
            },
            local_id: rustc_hir::hir_id::ItemLocalId::from_u32(layout.index),
        },
    }
}
