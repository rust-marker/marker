fn main() {
    let mut something = Some(12);

    #[warn(marker::marker_uilints::print_every_expr)]
    while let Some(_) = something {
        something = None;
    }
}
