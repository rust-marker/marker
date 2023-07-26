fn main() {
    let a = "Hey".to_string();

    let _print_simple_closure = || {
        1 + 1;
    };
    let _print_no_type: fn(u32) -> () = |x| { /*...*/ };
    let _print_with_args = |x: u32, y: u32| x + y;
    let _print_move = move || a;
    let _print_pattern_in_arg: fn((u32, u32, u32)) -> () = |(a, b, c)| { /*...*/ };
    // Make sure the infer type stays, if it originates from the source code
    let _print_infer_ty: fn(u32) -> () = |x: _| { /*...*/ };
}
