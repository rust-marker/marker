#[derive(Debug, Default)]
pub struct Example {
    a: u32,
}

impl Example {
    fn inc(&mut self, b: u32) {
        self.a += b;
    }

    fn print(&self) {
        println!("{self:#?}");
    }
}

fn main() {
    let mut foo = Example::default();
    let _print_method = foo.print();
    let _print_method = foo.inc(2);
}
