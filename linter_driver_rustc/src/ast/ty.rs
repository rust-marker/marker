use linter_api::ast::{
    ty::{Mutability, NumericKind, TextualKind, Ty, TyId, TyKind},
    CrateId,
};

use crate::ast::lifetime_from_region;

use super::{lifetime_from_hir, rustc::RustcContext, ToApi};

#[derive(Debug, Default, Clone)]
struct TyConvContext {
    pub is_infered: bool,
    // FIXME Generic store thingy
}

/// This trait is used to convert types like the normal [`ToApi`] trait. Additionally
/// it carries the current [`TyConvContext`] used to pass information from one type to
/// it's children. It's only intendet to be used inside this module
trait ToApiTy<'ast, 'tcx, T> {
    fn to_api_ty(&self, _cx: &'ast RustcContext<'ast, 'tcx>, _tccx: &mut TyConvContext) -> T;
}

// ========================================================
// General conversion
// ========================================================

impl<'ast, 'tcx> ToApi<'ast, 'tcx, TyId> for rustc_hir::def_id::DefId {
    fn to_api(&self, cx: &'ast RustcContext<'ast, 'tcx>) -> TyId {
        TyId::new(self.krate.to_api(cx), self.index.as_u32())
    }
}

impl<'ast, 'tcx> ToApiTy<'ast, 'tcx, TyId> for rustc_hir::def_id::DefId {
    fn to_api_ty(&self, cx: &'ast RustcContext<'ast, 'tcx>, _tyccx: &mut TyConvContext) -> TyId {
        self.to_api(cx)
    }
}

// ========================================================
// HIR conversion
// ========================================================

impl<'ast, 'tcx> ToApi<'ast, 'tcx, Mutability> for rustc_ast::ast::Mutability {
    fn to_api(&self, _cx: &'ast RustcContext<'ast, 'tcx>) -> Mutability {
        match self {
            rustc_middle::mir::Mutability::Mut => Mutability::Mut,
            rustc_middle::mir::Mutability::Not => Mutability::Not,
        }
    }
}

impl<'ast, 'tcx> ToApiTy<'ast, 'tcx, NumericKind> for rustc_ast::ast::IntTy {
    fn to_api_ty(&self, _cx: &'ast RustcContext<'ast, 'tcx>, _tyccx: &mut TyConvContext) -> NumericKind {
        match self {
            rustc_ast::ast::IntTy::Isize => NumericKind::Isize,
            rustc_ast::ast::IntTy::I8 => NumericKind::I8,
            rustc_ast::ast::IntTy::I16 => NumericKind::I16,
            rustc_ast::ast::IntTy::I32 => NumericKind::I32,
            rustc_ast::ast::IntTy::I64 => NumericKind::I64,
            rustc_ast::ast::IntTy::I128 => NumericKind::I128,
        }
    }
}

impl<'ast, 'tcx> ToApiTy<'ast, 'tcx, NumericKind> for rustc_ast::ast::UintTy {
    fn to_api_ty(&self, _cx: &'ast RustcContext<'ast, 'tcx>, _tyccx: &mut TyConvContext) -> NumericKind {
        match self {
            rustc_ast::ast::UintTy::Usize => NumericKind::Usize,
            rustc_ast::ast::UintTy::U8 => NumericKind::U8,
            rustc_ast::ast::UintTy::U16 => NumericKind::U16,
            rustc_ast::ast::UintTy::U32 => NumericKind::U32,
            rustc_ast::ast::UintTy::U64 => NumericKind::U64,
            rustc_ast::ast::UintTy::U128 => NumericKind::U128,
        }
    }
}

impl<'ast, 'tcx> ToApiTy<'ast, 'tcx, NumericKind> for rustc_ast::ast::FloatTy {
    fn to_api_ty(&self, _cx: &'ast RustcContext<'ast, 'tcx>, _tyccx: &mut TyConvContext) -> NumericKind {
        match self {
            rustc_ast::ast::FloatTy::F32 => NumericKind::F32,
            rustc_ast::ast::FloatTy::F64 => NumericKind::F64,
        }
    }
}

impl<'ast, 'tcx> ToApiTy<'ast, 'tcx, TyKind<'ast>> for rustc_hir::PrimTy {
    fn to_api_ty(&self, cx: &'ast RustcContext<'ast, 'tcx>, tyccx: &mut TyConvContext) -> TyKind<'ast> {
        match self {
            rustc_hir::PrimTy::Int(inner) => TyKind::Numeric(inner.to_api_ty(cx, tyccx)),
            rustc_hir::PrimTy::Uint(inner) => TyKind::Numeric(inner.to_api_ty(cx, tyccx)),
            rustc_hir::PrimTy::Float(inner) => TyKind::Numeric(inner.to_api_ty(cx, tyccx)),
            rustc_hir::PrimTy::Str => TyKind::Textual(TextualKind::Str),
            rustc_hir::PrimTy::Char => TyKind::Textual(TextualKind::Char),
            rustc_hir::PrimTy::Bool => TyKind::Bool,
        }
    }
}

impl<'ast, 'tcx> ToApiTy<'ast, 'tcx, TyKind<'ast>> for rustc_hir::Path<'tcx> {
    fn to_api_ty(&self, cx: &'ast RustcContext<'ast, 'tcx>, tyccx: &mut TyConvContext) -> TyKind<'ast> {
        match self.res {
            rustc_hir::def::Res::Def(def_kind, def_id) => {
                // This conversion can't be moved into a separate [`ToApi`]
                // Implementation as we need some context information.
                match def_kind {
                    rustc_hir::def::DefKind::Union
                    | rustc_hir::def::DefKind::Enum
                    | rustc_hir::def::DefKind::Struct => TyKind::Adt(def_id.to_api_ty(cx, tyccx)),
                    rustc_hir::def::DefKind::TyAlias => TyKind::TyAlias(def_id.to_api_ty(cx, tyccx)),
                    rustc_hir::def::DefKind::ForeignTy => TyKind::ForeignTy(def_id.to_api_ty(cx, tyccx)),
                    rustc_hir::def::DefKind::OpaqueTy => TyKind::ImplTrait(def_id.to_api_ty(cx, tyccx)),
                    rustc_hir::def::DefKind::TyParam => panic!("{self:#?}"),
                    // FIXME Add rustc_hir::def::DefKind::ConstParam,
                    // FIXME Add rustc_hir::def::DefKind::LifetimeParam,
                    rustc_hir::def::DefKind::Mod
                    | rustc_hir::def::DefKind::Variant
                    | rustc_hir::def::DefKind::TraitAlias
                    | rustc_hir::def::DefKind::AssocTy
                    | rustc_hir::def::DefKind::Fn
                    | rustc_hir::def::DefKind::Const
                    | rustc_hir::def::DefKind::Static(..)
                    | rustc_hir::def::DefKind::Ctor(..)
                    | rustc_hir::def::DefKind::AssocFn
                    | rustc_hir::def::DefKind::AssocConst
                    | rustc_hir::def::DefKind::Macro(..)
                    | rustc_hir::def::DefKind::ExternCrate
                    | rustc_hir::def::DefKind::Use
                    | rustc_hir::def::DefKind::ForeignMod
                    | rustc_hir::def::DefKind::AnonConst
                    | rustc_hir::def::DefKind::InlineConst
                    | rustc_hir::def::DefKind::Field
                    | rustc_hir::def::DefKind::GlobalAsm
                    | rustc_hir::def::DefKind::Impl
                    | rustc_hir::def::DefKind::Closure
                    | rustc_hir::def::DefKind::Generator
                    | rustc_hir::def::DefKind::Trait => panic!("this `DefKind` should not exist for {self:#?}"),
                    _ => unimplemented!(),
                }
            },
            rustc_hir::def::Res::PrimTy(prim) => prim.to_api_ty(cx, tyccx),
            // rustc_hir::def::Res::SelfTy { .. },
            // rustc_hir::def::Res::ToolMod,
            // rustc_hir::def::Res::SelfCtor(DefId),
            // rustc_hir::def::Res::Local(Id),
            // rustc_hir::def::Res::NonMacroAttr(NonMacroAttrKind),
            rustc_hir::def::Res::Err => unreachable!("at this stage all resources have been resolved"),
            _ => todo!(),
        }
    }
}

impl<'ast, 'tcx> ToApiTy<'ast, 'tcx, TyKind<'ast>> for rustc_hir::QPath<'tcx> {
    fn to_api_ty(&self, cx: &'ast RustcContext<'ast, 'tcx>, tyccx: &mut TyConvContext) -> TyKind<'ast> {
        match self {
            rustc_hir::QPath::Resolved(self_ty, path) => {
                assert!(self_ty.is_none(), "this should be `None` for types {self_ty:#?}");
                path.to_api_ty(cx, tyccx)
            },
            // rustc_hir::QPath::TypeRelative(&'hir Ty<'hir>, &'hir PathSegment<'hir>),
            // rustc_hir::QPath::LangItem(LangItem, Span, Option<HirId>),
            _ => unimplemented!(),
        }
    }
}

impl<'ast, 'tcx> ToApiTy<'ast, 'tcx, &'ast dyn Ty<'ast>> for rustc_hir::Ty<'tcx> {
    fn to_api_ty(&self, cx: &'ast RustcContext<'ast, 'tcx>, tyccx: &mut TyConvContext) -> &'ast dyn Ty<'ast> {
        let kind = match &self.kind {
            rustc_hir::TyKind::Slice(r_ty) => TyKind::Slice(r_ty.to_api_ty(cx, tyccx)),
            rustc_hir::TyKind::Array(r_ty, _fixme) => TyKind::Array(r_ty.to_api_ty(cx, tyccx)),
            rustc_hir::TyKind::Ptr(mut_ty) => TyKind::RawPtr(mut_ty.ty.to_api_ty(cx, tyccx), mut_ty.mutbl.to_api(cx)),
            rustc_hir::TyKind::Rptr(r_lt, mut_ty) => TyKind::Ref(
                mut_ty.ty.to_api_ty(cx, tyccx),
                mut_ty.mutbl.to_api(cx),
                lifetime_from_hir(cx, *r_lt),
            ),
            // rustc_hir::TyKind::BareFn(&'hir BareFnTy<'hir>),
            rustc_hir::TyKind::Never => TyKind::Never,
            rustc_hir::TyKind::Tup(lst) => {
                let ty_lst = cx.alloc_slice_from_iter(lst.iter().map(|r_ty| r_ty.to_api_ty(cx, tyccx)));
                TyKind::Tuple(ty_lst)
            },
            rustc_hir::TyKind::Path(qpath) => qpath.to_api_ty(cx, tyccx),
            // rustc_hir::TyKind::OpaqueDef(ItemId, &'hir [GenericArg<'hir>]),
            // rustc_hir::TyKind::TraitObject(&'hir [PolyTraitRef<'hir>], Lifetime, TraitObjectSyntax),
            // rustc_hir::TyKind::Typeof(AnonConst),
            rustc_hir::TyKind::Infer => {
                tyccx.is_infered = true;
                todo!()
            },
            rustc_hir::TyKind::Err => unreachable!(),
            _ => todo!(),
        };

        // FIXME: Set infer correctly
        cx.alloc_with(|| RustcTy::new(cx, kind, tyccx.is_infered))
    }
}

impl<'ast, 'tcx> ToApi<'ast, 'tcx, &'ast dyn Ty<'ast>> for rustc_hir::Ty<'tcx> {
    fn to_api(&self, cx: &'ast RustcContext<'ast, 'tcx>) -> &'ast dyn Ty<'ast> {
        self.to_api_ty(cx, &mut TyConvContext::default())
    }
}

// ============================================================================
// Prototype:
// ============================================================================

pub struct RustcTy<'ast, 'tcx> {
    pub cx: &'ast RustcContext<'ast, 'tcx>,
    pub kind: TyKind<'ast>,
    pub is_infered: bool,
}

impl<'ast, 'tcx> RustcTy<'ast, 'tcx> {
    #[must_use]
    pub fn new(cx: &'ast RustcContext<'ast, 'tcx>, kind: TyKind<'ast>, is_infered: bool) -> Self {
        Self { cx, kind, is_infered }
    }

    #[must_use]
    pub fn from_rustc_hir_ty(
        cx: &'ast RustcContext<'ast, 'tcx>,
        hir_ty: &'tcx rustc_hir::Ty<'tcx>,
    ) -> &'ast dyn Ty<'ast> {
        hir_ty.to_api(cx)
    }
}

impl<'ast, 'tcx> std::fmt::Debug for RustcTy<'ast, 'tcx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustcTy")
            .field("kind", &self.kind)
            .field("is_infered", &self.is_infered)
            .finish()
    }
}

impl<'ast, 'tcx> Ty<'ast> for RustcTy<'ast, 'tcx> {
    fn get_kind(&'ast self) -> &TyKind<'ast> {
        &self.kind
    }

    fn is_infered(&self) -> bool {
        self.is_infered
    }
}

fn ty_id_from_def_id(did: rustc_span::def_id::DefId) -> TyId {
    TyId::new(CrateId::new(did.krate.as_u32()), did.index.as_u32())
}

pub fn create_from_rustc_ty<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    rustc_ty: rustc_middle::ty::Ty<'tcx>,
) -> &'ast dyn Ty<'ast> {
    let kind = match rustc_ty.kind() {
        rustc_middle::ty::TyKind::Bool => TyKind::Bool,
        rustc_middle::ty::TyKind::Char => TyKind::Textual(TextualKind::Char),
        rustc_middle::ty::TyKind::Str => TyKind::Textual(TextualKind::Str),
        rustc_middle::ty::TyKind::Int(int_ty) => {
            let numeric_kind = match int_ty {
                rustc_middle::ty::IntTy::Isize => NumericKind::Isize,
                rustc_middle::ty::IntTy::I8 => NumericKind::I8,
                rustc_middle::ty::IntTy::I16 => NumericKind::I16,
                rustc_middle::ty::IntTy::I32 => NumericKind::I32,
                rustc_middle::ty::IntTy::I64 => NumericKind::I64,
                rustc_middle::ty::IntTy::I128 => NumericKind::I128,
            };

            TyKind::Numeric(numeric_kind)
        },
        rustc_middle::ty::TyKind::Uint(uint_ty) => {
            let numeric_kind = match uint_ty {
                rustc_middle::ty::UintTy::Usize => NumericKind::Usize,
                rustc_middle::ty::UintTy::U8 => NumericKind::U8,
                rustc_middle::ty::UintTy::U16 => NumericKind::U16,
                rustc_middle::ty::UintTy::U32 => NumericKind::U32,
                rustc_middle::ty::UintTy::U64 => NumericKind::U64,
                rustc_middle::ty::UintTy::U128 => NumericKind::U128,
            };

            TyKind::Numeric(numeric_kind)
        },
        rustc_middle::ty::TyKind::Float(float_ty) => {
            let numeric_kind = match float_ty {
                rustc_middle::ty::FloatTy::F32 => NumericKind::F32,
                rustc_middle::ty::FloatTy::F64 => NumericKind::F64,
            };

            TyKind::Numeric(numeric_kind)
        },
        rustc_middle::ty::TyKind::Never => TyKind::Never,
        rustc_middle::ty::TyKind::Tuple(lst) => {
            let ty_lst = cx.alloc_slice_from_iter(lst.iter().map(|rustc_ty| create_from_rustc_ty(cx, rustc_ty)));
            TyKind::Tuple(ty_lst)
        },
        rustc_middle::ty::TyKind::Array(r_ty, _fixme) => TyKind::Array(create_from_rustc_ty(cx, *r_ty)),
        rustc_middle::ty::TyKind::Slice(r_ty) => TyKind::Slice(create_from_rustc_ty(cx, *r_ty)),

        rustc_middle::ty::TyKind::Adt(adt, _) => TyKind::Adt(ty_id_from_def_id(adt.did())),
        rustc_middle::ty::TyKind::Foreign(did) => TyKind::Adt(ty_id_from_def_id(*did)),
        rustc_middle::ty::TyKind::RawPtr(ty_and_mut) => {
            TyKind::RawPtr(create_from_rustc_ty(cx, ty_and_mut.ty), ty_and_mut.mutbl.to_api(cx))
        },
        rustc_middle::ty::TyKind::Ref(r_lt, r_ty, r_mut) => TyKind::Ref(
            create_from_rustc_ty(cx, *r_ty),
            r_mut.to_api(cx),
            lifetime_from_region(cx, *r_lt),
        ),

        // rustc_middle::ty::TyKind::Opaque(DefId, SubstsRef<'tcx>),
        // ImplTrait
        // rustc_middle::ty::TyKind::Dynamic(&'tcx List<Binder<'tcx, ExistentialPredicate<'tcx>>>, ty::Region<'tcx>),
        // DynTrait

        // rustc_middle::ty::TyKind::FnDef(DefId, SubstsRef<'tcx>),
        // rustc_middle::ty::TyKind::FnPtr(PolyFnSig<'tcx>),
        // rustc_middle::ty::TyKind::Closure(DefId, SubstsRef<'tcx>),
        // rustc_middle::ty::TyKind::Generator(DefId, SubstsRef<'tcx>, hir::Movability),
        // rustc_middle::ty::TyKind::GeneratorWitness(Binder<'tcx, &'tcx List<Ty<'tcx>>>),
        // rustc_middle::ty::TyKind::Projection(ProjectionTy<'tcx>),
        // rustc_middle::ty::TyKind::Param(ParamTy),
        // rustc_middle::ty::TyKind::Bound(ty::DebruijnIndex, BoundTy),
        rustc_middle::ty::TyKind::Placeholder(_)
        | rustc_middle::ty::TyKind::Infer(_)
        | rustc_middle::ty::TyKind::Error(_) => unreachable!(),
        _ => todo!(),
    };

    // These types are never infered as they are created from the exect rustc type
    cx.new_ty(kind, false)
}
