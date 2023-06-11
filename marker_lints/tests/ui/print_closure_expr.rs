// normalize-stderr-windows: "tests/ui/" -> "$$DIR/"

fn main() {
    let a = "Hey".to_string();

    let _print_simple_closure = || {
        1 + 1;
    };
    let _print_with_args = |x: u32, y: u32| x + y;
    let _print_move = move || a;
}
