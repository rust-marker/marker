fn main() {
    #[warn(marker::marker_uilints::print_every_expr)]
    let _ = 1..2;
    #[warn(marker::marker_uilints::print_every_expr)]
    let _ = ..2;
    #[warn(marker::marker_uilints::print_every_expr)]
    let _ = 1..;
    #[warn(marker::marker_uilints::print_every_expr)]
    let _ = ..;
}
