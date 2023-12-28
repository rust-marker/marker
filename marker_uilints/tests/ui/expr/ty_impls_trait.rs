#![warn(marker::marker_uilints::test_ty_impls_trait)]
use std::fmt::Debug;

trait SimpleTestTrait {}
impl SimpleTestTrait for String {}

trait GenericTestTrait<T: Debug> {}
impl GenericTestTrait<i32> for String {}

trait AssocTyTestTrait {
    type Assoc: Debug;
}
impl AssocTyTestTrait for String {
    type Assoc=i32;
}

fn check_generic<T: Clone>(check_traits_from_generic: &T) {
    let _ = check_traits_from_generic.clone();
}

fn check_more_generics<T: Ord>(check_traits_ord: T, y: T, z: T) {
    if check_traits_ord > y {
        todo!("a");
    } else if z < y {
        todo!("b");
    }
}

fn return_impl_clone() -> impl Clone {
    "I'm a beautiful string, that implements `Clone`"
}

fn check_impl_ret() {
    let check_traits_for_impl_clone = return_impl_clone();
    let _ = check_traits_for_impl_clone;
}

fn main() {
    let check_traits_i32 = 1;
    let _ = check_traits_i32;

    let check_traits_string = String::new();
    let _ = check_traits_string;
}
