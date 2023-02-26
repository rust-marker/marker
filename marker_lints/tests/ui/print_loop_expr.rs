// normalize-stdout-windows: "tests/ui/" -> "$$DIR/"

fn main() {
    let cond = true;
    let _print_exprs = {
        while cond {}

        for _ in 0..10 {}

        loop {}
    };
}