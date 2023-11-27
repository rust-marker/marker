#![warn(marker::marker_uilints::test_ty_impls_trait)]

fn check_generic<T: Clone>(check_traits_from_generic: &T) {
    let _ = check_traits_from_generic.clone();
}

fn return_impl_clone() -> impl Clone {
    "I'm a beautiful string, that implements `Clone`"
}

fn check_impl_ret() {
    let check_traits_for_impl_clone = return_impl_clone();
    let _ = check_traits_for_impl_clone;
}

fn main() {
    let check_traits = 1;
    let _ = check_traits;
}