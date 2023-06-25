// normalize-stderr-windows: "tests/ui/" -> "$$DIR/"

#[derive(Default)]
struct FieldStruct {
    a: u32,
}

fn main() {
    let mut object = FieldStruct { a: 1 };
    let tuple = (1, 2);
    let array = [1, 2, 3];

    let _print_struct_field = object.a;
    let _print_tuple_field = tuple.0;
    let _print_array_index = array[0];
}
