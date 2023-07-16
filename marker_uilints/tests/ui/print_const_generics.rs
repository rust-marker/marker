
struct PrintMeConstGenerics<const N: usize> {
    data: [f32; N],
}

fn print_me() -> PrintMeConstGenerics<3> {
    todo!()
}

impl<const N: usize> PrintMeConstGenerics<N> {}

fn main() {
    let _ty: PrintMeConstGenerics<3> = PrintMeConstGenerics {
        data: [1.0, 1.5, 2.0],
    };
}
