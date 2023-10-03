use marker_api::sem::{BindingArg, ConstArg, ConstValue, GenericArgKind, GenericArgs, TraitBound};
use rustc_middle as mid;

use crate::conversion::marker::MarkerConverterInner;

impl<'ast, 'tcx> MarkerConverterInner<'ast, 'tcx> {
    #[must_use]
    pub fn to_sem_generic_args(&self, args: &[mid::ty::GenericArg<'tcx>]) -> GenericArgs<'ast> {
        let args: Vec<_> = args
            .iter()
            .filter_map(|arg| self.to_sem_generic_arg_kind(*arg))
            .collect();

        GenericArgs::new(self.alloc_slice(args))
    }

    #[must_use]
    fn to_sem_generic_arg_kind(&self, arg: mid::ty::GenericArg<'tcx>) -> Option<GenericArgKind<'ast>> {
        match &arg.unpack() {
            mid::ty::GenericArgKind::Lifetime(_) => None,
            mid::ty::GenericArgKind::Type(ty) => Some(GenericArgKind::Ty(self.to_sem_ty(*ty))),
            mid::ty::GenericArgKind::Const(_) => {
                Some(GenericArgKind::Const(self.alloc(ConstArg::new(ConstValue::new()))))
            },
        }
    }

    pub fn to_sem_trait_bounds(
        &self,
        bounds: &mid::ty::List<mid::ty::PolyExistentialPredicate<'tcx>>,
    ) -> &'ast [TraitBound<'ast>] {
        let mut marker_bounds = vec![];

        // Understanding this representation, was a journey of at least 1.5 liters
        // of tea, way too many print statements and complaining to a friend of mine.
        //
        // Here is the basic breakdown:
        // * Due to [`E0225`] these bounds are currently restricted to one *main* trait. Any other traits
        //   have to be auto traits.
        // * Simple generic args, like the `u32` in `Trait<u32>`, are stored in the `substs` of the trait.
        // * Named type parameters, like `Item = u32` in `dyn Iterator<Item = u32>`, are stored as
        //   `ExistentialPredicate::Projection` in the list of bindings. These parameters now need to be
        //   *reattached* to the `SemGenericArgs` of the *main* trait, to work with markers representation.
        //
        // [`E0225`]: https://doc.rust-lang.org/stable/error_codes/E0225.html
        if let Some(main) = bounds.principal() {
            let main = main.skip_binder();

            let mut generics: Vec<_> = main
                .args
                .iter()
                .filter_map(|arg| self.to_sem_generic_arg_kind(arg))
                .collect();

            bounds
                .projection_bounds()
                .for_each(|binding| match binding.skip_binder().term.unpack() {
                    mid::ty::TermKind::Ty(ty) => generics.push(GenericArgKind::Binding(self.alloc(BindingArg::new(
                        self.to_item_id(binding.item_def_id()),
                        self.to_sem_ty(ty),
                    )))),
                    mid::ty::TermKind::Const(_) => todo!(),
                });

            marker_bounds.push(TraitBound::new(
                false,
                self.to_ty_def_id(main.def_id),
                GenericArgs::new(self.alloc_slice(generics)),
            ));
        }

        bounds
            .auto_traits()
            .map(|auto_trait_id| {
                TraitBound::new(false, self.to_ty_def_id(auto_trait_id), self.to_sem_generic_args(&[]))
            })
            .collect_into(&mut marker_bounds);

        self.alloc_slice(marker_bounds)
    }
}
