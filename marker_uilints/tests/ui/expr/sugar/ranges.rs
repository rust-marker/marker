#![feature(register_tool)]
#![register_tool(marker)]

fn main() {
    #[warn(marker::print_every_expr)]
    let _ = 1..2;
    #[warn(marker::print_every_expr)]
    let _ = ..2;
    #[warn(marker::print_every_expr)]
    let _ = 1..;
    #[warn(marker::print_every_expr)]
    let _ = ..;
}
