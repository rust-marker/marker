#![allow(unused)]

fn test_contains_return_1_false() {
    // False, doesn't contain anything
}

fn test_contains_return_2_false() {
    if (16 << 2) < 2 {
        println!("IDK");
    } else {
        println!("idk");
    }
}

fn test_contains_return_3_true() {
    // True, is this code useful? Nope, but it contains a return
    return;
}

fn test_contains_return_4_true() -> Option<u32> {
    // True, is this code useful? Still nope, somehow it's worse
    let x: u32 = Some(2)?;
    Some(x)
}

fn test_contains_return_5_false() {
    // False, the return is nested
    fn nested_function() {
        return;
    }
    nested_function();
}
