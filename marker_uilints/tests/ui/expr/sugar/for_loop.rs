#![feature(register_tool)]
#![register_tool(marker)]

fn main() {
    let range = 1..10;
    let mut total = 0;

    #[warn(marker::print_every_expr)]
    for i in range {
        total += i;
    }

    println!("{total}");
}
