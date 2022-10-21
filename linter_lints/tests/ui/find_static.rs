use core::fmt::Debug;
use core::marker::PhantomData;

// static TEST: (u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, &str, Option<Vec<Dog>>, Option<(&[f32], [i32; 7], char, bool, Cat<*const i32>, Cat<Box<dyn Debug + 'static>>)>) = (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "Duck", None, None);

#[clippy::dump]
static TEST: Option<(
    Cat<Box<dyn Debug>>,
    Cat<Box<dyn Iterator<Item=i32> + 'static>>
)> = None;

pub union Dog {
    _f: f32,
    _i: i32
}

pub struct Cat<T> {
    _data: PhantomData<T>,
}
unsafe impl<T> Sync for Cat<T> {}

fn main() {}
