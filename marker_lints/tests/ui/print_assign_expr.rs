// normalize-stdout-windows: "tests/ui/" -> "$$DIR/"

fn bar() -> i32 {
    16
}

pub fn main() {
    let mut a = 0;
    let mut b = 0;
    let _print_exprs = {
        a = bar();
        a += 1;
        [a, b] = [1, 2];
        ()
    };
}
