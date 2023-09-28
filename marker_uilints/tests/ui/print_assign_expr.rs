#[derive(Debug, Default)]
struct S {
    array: [i32; 3],
    slice: (i32, i32, i32),
}

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
        S {
            array: [_, a, ..],
            slice: (b, ..),
        } = S::default();
        ()
    };
}
