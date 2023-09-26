enum LocalOption<T> {
    MySome(T),
    MyNone,
}

struct LocalStruct {
    data: u32,
}

fn main() {
    let _check_ast_map = Option::Some(12);
    let _check_ast_map = LocalOption::MySome(17);

    let _check_ast_map = LocalStruct { data: 17 };
}
