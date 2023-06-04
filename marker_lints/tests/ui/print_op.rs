// normalize-stderr-windows: "tests/ui/" -> "$$DIR/"

fn main() {
    let mut value = 20;
    let _print_alg_ops = 1 + 2 * -3;
    let _print_bool_ops = true && false || !true;
    let _print_ref = &mut value;
}
