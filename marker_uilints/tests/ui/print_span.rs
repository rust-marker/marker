macro_rules! magic_macro {
    () => {
        "*magic penguin noises*"
    };
}
use magic_macro;

fn try_something() -> Option<u32> {
    let _span_try = Some(21)?;
    None
}

fn main() {
    let _span_normal = 178;

    let _span_macro = magic_macro!();
}
