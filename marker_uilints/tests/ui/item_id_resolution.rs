struct TestType(u32);

// Please don't change the function name, it's used by the lint
fn test_ty_id_resolution_trigger() {
    let _check_path_vec = vec!["hey"];
    let _check_path_string = String::from("marker");
    let _check_path_option = Option::Some("<3");
    let _check_path_test_type = TestType(3);
}

fn main() {}
