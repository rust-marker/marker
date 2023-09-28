#![feature(register_tool)]
#![register_tool(marker)]

fn try_something() -> Option<u32> {
    #[warn(marker::print_every_expr)]
    Some(21)?;
    None
}

fn kanske_option() -> Result<i32, u32> {
    let x: Result<i32, u32> = Err(1);
    #[warn(marker::print_every_expr)]
    x?;
    Ok(42)
}

fn main() {}
