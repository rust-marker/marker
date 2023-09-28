pub fn print_me_simple() {}

pub const unsafe fn print_me_special() {}

pub fn print_me_params(_ident: u32, (_a, _b): (u32, i32)) -> String {
    String::new()
}

pub trait SomethingCool {
    fn print_me_trait_with_body(_ident: u8, (_a, _b): (u8, i8)) -> String {
        String::new()
    }

    fn print_me_trait_no_body(_ident: u64, _pair: (u64, i64)) -> String;
}

fn main() {}
