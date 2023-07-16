macro_rules! magic_macro {
    () => {
        "*magic penguin noises*"
    };
}
use magic_macro;

fn main() {
    let _span_normal = 178;

    let _span_macro = magic_macro!();
}
