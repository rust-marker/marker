// normalize-stderr-windows: "tests/ui/" -> "$$DIR/"

use core::fmt::Debug;
use core::marker::PhantomData;

static HELP_ITEM_TYPE_SEQUENCE: Option<AllowSync<(&[i32], [i32; 8])>> = None;
static HELP_ITEM_TYPE_POINTER: Option<AllowSync<(&'static str, *const i32, *mut i32)>> = None;
static HELP_ITEM_TYPE_COMPLEX: Option<
    AllowSync<(
        AliasTy,
        String,
        Option<String>,
        Vec<UnionItem>,
        Box<dyn Debug>,
        Box<dyn Iterator<Item = i32> + 'static>,
    )>,
> = None;

type AliasTy = Box<u32>;

pub union UnionItem {
    _f: f32,
    _i: i32,
}

pub struct AllowSync<T> {
    _data: PhantomData<T>,
}
unsafe impl<T> Sync for AllowSync<T> {}

trait InterestingTrait<T> {
    type A: Default;
    fn use_alias(&self) {
        // FIXME: This expression is currently not found by the print test code.
        // Try to figure out why and then make sure that is is correctly represented.
        let _ty: Self::A = Self::A::default();
    }
}

fn param_type<T: Debug>(t: T) {
    let _ty_generic: T = t;
}

fn main() {
    let _ty: u32 = 10;
    let _ty_primitive: Option<(u8, u16, u32, u64, u128, usize)> = None;
    let _ty_primitive: Option<(i8, i16, i32, i64, i128, isize)> = None;
    let _ty_primitive: Option<(char, bool, f32, f64)> = None;
    let _ty_sequence: [u32; 1] = [10];
    let slice: &[u32] = &[10];
    let _ty_sequence: &[u32] = slice;
    let _ty_ptr: Option<(&'static str, *const i32, *mut i32)> = None;

    // Interestingly, rustc substitutes the type directly and the semantic type
    // doesn't show the type alias.
    let _ty_simple_alias: AliasTy = AliasTy::new(12);

    let _ty_adt: String = String::new();
    let _ty_dyn_simple: Option<Box<dyn Debug>> = None;
    let _ty_dyn_complex: Option<Box<dyn Iterator<Item = i32> + 'static>> = None;
}
