async fn foo() -> u8 {
    16
}

async fn bar() {
    let _print_await = foo().await;

    let future = foo();
    let _print_await = future.await;

    let _print_await = async { 18 }.await;
}

fn main() {}
