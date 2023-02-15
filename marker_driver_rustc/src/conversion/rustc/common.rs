use std::mem::{size_of, transmute};

use marker_api::{
    ast::{BodyId, CrateId, GenericId, ItemId, Span, SpanId, SymbolId, TyDefId},
    lint::Level,
};
use rustc_hir as hir;

use crate::conversion::common::{BodyIdLayout, DefIdInfo, GenericIdLayout, ItemIdLayout, TyDefIdLayout};
use crate::transmute_id;

use super::RustcConverter;

macro_rules! impl_into_def_id_for {
    ($id:ty, $layout:ty) => {
        impl From<$id> for DefIdInfo {
            fn from(value: $id) -> Self {
                let layout = transmute_id!($id as $layout = value);
                DefIdInfo {
                    index: layout.index,
                    krate: layout.krate,
                }
            }
        }
    };
}

use impl_into_def_id_for;

impl_into_def_id_for!(GenericId, GenericIdLayout);
impl_into_def_id_for!(ItemId, ItemIdLayout);
impl_into_def_id_for!(TyDefId, TyDefIdLayout);

#[derive(Debug, Clone, Copy)]
pub struct SpanSourceInfo {
    pub rustc_span_cx: rustc_span::hygiene::SyntaxContext,
    pub rustc_start_offset: usize,
}

impl<'ast, 'tcx> RustcConverter<'ast, 'tcx> {
    #[must_use]
    pub fn to_crate_num(&self, api_id: CrateId) -> hir::def_id::CrateNum {
        assert_eq!(size_of::<CrateId>(), 4);
        hir::def_id::CrateNum::from_u32(api_id.data())
    }

    #[must_use]
    pub fn to_item_id(&self, api_id: ItemId) -> hir::ItemId {
        let layout = transmute_id!(ItemId as ItemIdLayout = api_id);
        hir::ItemId {
            owner_id: hir::OwnerId {
                def_id: hir::def_id::LocalDefId {
                    local_def_index: hir::def_id::DefIndex::from_u32(layout.index),
                },
            },
        }
    }

    #[must_use]
    pub fn to_body_id(&self, api_id: BodyId) -> hir::BodyId {
        let layout = transmute_id!(BodyId as BodyIdLayout = api_id);
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
    pub fn to_symbol(&self, api_id: SymbolId) -> rustc_span::Symbol {
        assert_eq!(size_of::<SymbolId>(), 4);
        assert_eq!(size_of::<rustc_span::Symbol>(), 4);
        // FIXME: `rustc_span::Symbol` currently has no public constructor for the
        // index value and no `#[repr(C)]` attribute. Therefore, this conversion is
        // unsound. This requires changes in rustc.
        unsafe { transmute(api_id) }
    }

    #[must_use]
    pub fn to_span_from_id(&self, api_id: SpanId) -> rustc_span::Span {
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
    pub fn to_def_id(api_id: impl Into<DefIdInfo>) -> hir::def_id::DefId {
        let info: DefIdInfo = api_id.into();
        hir::def_id::DefId {
            index: hir::def_id::DefIndex::from_u32(info.index),
            krate: hir::def_id::CrateNum::from_u32(info.krate),
        }
    }

    #[must_use]
    pub fn to_lint_level(&self, api_level: Level) -> rustc_lint::Level {
        match api_level {
            Level::Allow => rustc_lint::Level::Allow,
            Level::Warn => rustc_lint::Level::Warn,
            Level::Deny => rustc_lint::Level::Deny,
            Level::Forbid => rustc_lint::Level::Forbid,
            _ => unreachable!(),
        }
    }

    pub fn to_span(&self, api_span: &Span<'ast>) -> rustc_span::Span {
        let src_info = self
            .storage
            .span_src_info(api_span.source())
            .expect("all driver created `SpanSources` have a matching info");

        #[expect(clippy::cast_possible_truncation, reason = "`u32` is set by rustc and will be fine")]
        let lo = rustc_span::BytePos((api_span.start() + src_info.rustc_start_offset) as u32);
        #[expect(clippy::cast_possible_truncation, reason = "`u32` is set by rustc and will be fine")]
        let hi = rustc_span::BytePos((api_span.end() + src_info.rustc_start_offset) as u32);
        rustc_span::Span::new(lo, hi, src_info.rustc_span_cx, None)
    }
}
