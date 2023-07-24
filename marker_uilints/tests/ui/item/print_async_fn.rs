async fn print_with_body_foo() -> u8 {
    16
}

async fn foo() -> u8 {
    16
}

async fn print_with_body_bar() -> u8 {
    let a: u8 = foo().await;
    let b: u8 = foo().await;
    let c: u8 = foo().await;
    a + b + c
}