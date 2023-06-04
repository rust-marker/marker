// normalize-stderr-windows: "tests/ui/" -> "$$DIR/"

fn main() {
    let _print_exprs = {
        'label: loop {
            break 'label;
        }

        loop {}
    };

    let cond = true;
    let mut opt = Some(0xF);
    let _print_exprs = {
        while cond {}

        while let Some(_) = opt {
            opt = None;
        }
    };

    let tuple_slice = [(1, 2), (3, 4)];
    let mut c = 0;
    let _print_exprs = {
        for _ in 0..10 {}

        for (a, b) in tuple_slice {
            c += a + b;
        }
    };
}
