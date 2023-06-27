pub trait A {
    fn a() {
        println!("a()");
    }
}

pub trait B {
    type CoolTy: A;

    fn assoc_item_ty(cool: Self::CoolTy) -> Self::CoolTy {
        cool
    }
}

struct Magic;
struct Sparkles;

impl A for Sparkles {}

impl B for Magic {
    type CoolTy = Sparkles;
}

fn rand(_seed: u32) -> u16 {
    5
}

pub fn main() {
    let _print_path = rand(7);
    let _print_path = Vec::<u32>::new();
    let _print_path = <Magic as B>::CoolTy::a();
    let _print_path = <<Magic as B>::CoolTy as A>::a();

    let var = 1;
    let _print_path = var;
}
