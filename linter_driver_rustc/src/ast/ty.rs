use linter_api::ast::{
    ty::{Mutability, NumericKind, TextualKind, Ty, TyId, TyKind},
    CrateId,
};

use crate::ast::lifetime_from_region;

use super::{rustc::RustcContext, lifetime_from_hir};

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

fn mutability_from_rustc(rustc_mut: rustc_middle::mir::Mutability) -> Mutability {
    match rustc_mut {
        rustc_middle::mir::Mutability::Mut => Mutability::Mut,
        rustc_middle::mir::Mutability::Not => Mutability::Not,
    }
}

fn mutability_from_hir(hir_mut: rustc_hir::Mutability) -> Mutability {
    match hir_mut {
        rustc_hir::Mutability::Mut => Mutability::Mut,
        rustc_hir::Mutability::Not => Mutability::Not,
    }
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
        rustc_middle::ty::TyKind::RawPtr(ty_and_mut) => TyKind::RawPtr(
            create_from_rustc_ty(cx, ty_and_mut.ty),
            mutability_from_rustc(ty_and_mut.mutbl),
        ),
        rustc_middle::ty::TyKind::Ref(r_lt, r_ty, r_mut) => TyKind::Ref(
            create_from_rustc_ty(cx, *r_ty),
            mutability_from_rustc(*r_mut),
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

pub fn create_from_hir_ty<'ast, 'tcx>(
    cx: &'ast RustcContext<'ast, 'tcx>,
    rustc_ty: &rustc_hir::Ty<'tcx>,
) -> &'ast dyn Ty<'ast> {
    let kind = match &rustc_ty.kind {
        rustc_hir::TyKind::Slice(r_ty) => TyKind::Slice(create_from_hir_ty(cx, r_ty)),
        rustc_hir::TyKind::Array(r_ty, _fixme) => TyKind::Array(create_from_hir_ty(cx, r_ty)),
        rustc_hir::TyKind::Ptr(mut_ty) => TyKind::RawPtr(create_from_hir_ty(cx, mut_ty.ty), mutability_from_hir(mut_ty.mutbl)),
        rustc_hir::TyKind::Rptr(r_lt, mut_ty) => TyKind::Ref(
            create_from_hir_ty(cx, mut_ty.ty),
            mutability_from_hir(mut_ty.mutbl),
            lifetime_from_hir(cx, *r_lt),
        ),
        // rustc_hir::TyKind::BareFn(&'hir BareFnTy<'hir>),
        rustc_hir::TyKind::Never => TyKind::Never,
        rustc_hir::TyKind::Tup(lst) => {
            let ty_lst = cx.alloc_slice_from_iter(lst.iter().map(|rustc_ty| create_from_hir_ty(cx, rustc_ty)));
            TyKind::Tuple(ty_lst)
        },
        rustc_hir::TyKind::Path(qpath) => {
            match qpath {
                rustc_hir::QPath::Resolved(_opt_r_ty, path) => {
                    match path.res {
                        rustc_hir::def::Res::PrimTy(_prim) => {
                            todo!()
                        },
                        // Def(DefKind, DefId),
                        // SelfTy {
                        //     trait_: Option<DefId>,
                        //     alias_to: Option<(DefId, bool)>,
                        // },
                        // ToolMod,
                        // SelfCtor(DefId),
                        // Local(Id),
                        // NonMacroAttr(NonMacroAttrKind),
                        rustc_hir::def::Res::Err => unreachable!("At this point all types should be resolved"),
                        _ => todo!(),
                    }
                },
                // rustc_hir::QPath::TypeRelative(&'hir Ty<'hir>, &'hir PathSegment<'hir>),
                // rustc_hir::QPath::LangItem(LangItem, Span, Option<HirId>),
                _ => unimplemented!()
            }
        },
        // rustc_hir::TyKind::OpaqueDef(ItemId, &'hir [GenericArg<'hir>]),
        // rustc_hir::TyKind::TraitObject(&'hir [PolyTraitRef<'hir>], Lifetime, TraitObjectSyntax),
        // rustc_hir::TyKind::Typeof(AnonConst),
        // rustc_hir::TyKind::Infer,
        rustc_hir::TyKind::Err => unreachable!(),
        _ => todo!(),
    };

    // FIXME: Set infer correctly
    cx.new_ty(kind, false)
}
