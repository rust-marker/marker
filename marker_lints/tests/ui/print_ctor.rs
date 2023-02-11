// normalize-stdout-windows: "tests/ui/" -> "$$DIR/"

#[derive(Debug, Default)]
struct FieldStruct {
    a: u32,
    b: u32,
}
#[derive(Default)]
struct TupleStruct(u32, u32);
union Union {
    a: u32,
}
enum Enum {
    A,
    B(u32),
    C { f1: u32, f2: u32 },
}

fn main() {
    let _print_tuple = (1, 2, 3);
    let _print_array = [1, 2, 3];
    let _print_array = [1; 3];

    let _print_range = 11..;
    let _print_range = 1..3;
    let _print_range = ..3;
    let _print_range = ..=3;
    let _print_range = 1..=3;
    let _print_range = ..;

    let _print_ctor = FieldStruct { a: 1, b: 2 };
    let _print_ctor = FieldStruct { a: 10, ..FieldStruct::default() };

    let _print_ctor = Union { a: 8 };

    // let _ = TupleStruct(1, 2);
    let _print_ctor = TupleStruct { 0: 3, ..TupleStruct::default() };

    let _ = Enum::A;
    // let _ = Enum::B(1);
    // let _ = Enum::C { f1: 44, f2: 55 };
}