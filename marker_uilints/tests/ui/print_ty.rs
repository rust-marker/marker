use core::fmt::Debug;
use core::marker::PhantomData;

static PRINT_TYPE_PRIMITIVE_1: Option<(u8, u16, u32, u64, u128, usize)> = None;
static PRINT_TYPE_PRIMITIVE_2: Option<(i8, i16, i32, i64, i128, isize)> = None;
static PRINT_TYPE_PRIMITIVE_3: Option<(char, bool, f32, f64)> = None;
static PRINT_TYPE_SEQUENCE: Option<AllowSync<(&[i32], [i32; 8])>> = None;
static PRINT_TYPE_POINTER: Option<AllowSync<(&'static str, *const i32, *mut i32)>> = None;
static PRINT_TYPE_COMPLEX: Option<
    AllowSync<(
        AliasTy,
        String,
        Option<String>,
        Vec<UnionItem>,
        Box<dyn Debug>,
        Box<dyn Iterator<Item = i32> + 'static>,
    )>,
> = None;
static PRINT_TYPE_GROUPED_1: (u32) = 0;
static PRINT_TYPE_GROUPED_2: ((u32)) = 0;

type AliasTy = (u8, u16);

pub union UnionItem {
    _f: f32,
    _i: i32,
}

pub struct AllowSync<T> {
    _data: PhantomData<T>,
}
unsafe impl<T> Sync for AllowSync<T> {}

fn main() {}
