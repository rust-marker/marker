fn ifs() {
    let cond = true;
    let _print_if = if cond {
        // The simple if condition sadly has to be printed as a sub expression
        // as only `let _print*` triggers the print lint
        if cond {}
        1
    } else {
        2
    };

    let opt = Some(1);
    let _print_if_let = if let Some(_) = opt { "some" } else { "none" };

    let a = true;
    let b = true;
    let _print_else_if = if a {
        1
    } else if b {
        2
    } else {
        3
    };
}

fn matches(scrutinee: &[i32]) {
    let _print_match = match scrutinee {
        [] => 1,
        [x] if check(x) => 2,
        _ => {
            // A block as the arm expression
            3
        },
    };

    let opt = Some(1);
    let _print_match_with_path = match opt {
        Some(-1) => (),
        Some(1) => (),
        Some(x) => (),
        None => (),
    };
}

mod question_mark {
    fn kanske_option() -> Option<i32> {
        let x = Some(1);
        let _print_option_match = x?;
        None
    }

    fn kanske_result() -> Result<i32, i32> {
        let x: Result<i32, i32> = Ok(1);
        let _print_result_match = x?;
        Err(4)
    }
}

fn check(_: &i32) -> bool {
    true
}

fn main() {}
