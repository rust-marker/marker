#![cfg_attr(marker, feature(register_tool))]
#![cfg_attr(marker, register_tool(marker))]

mod allow_with_simple_attr {
    #[cfg_attr(marker, allow(marker::item_with_test_name))]
    fn find_me_fn() {}
}

mod allow_with_crate_check_attr {
    #[cfg_attr(marker = "marker_uilints", allow(marker::item_with_test_name))]
    fn find_me_fn() {}
}

mod lint_with_unloaded_crate_attr {
    #[cfg_attr(marker = "some_unknown_crate_that_isnt_loaded", allow(marker::item_with_test_name))]
    fn find_me_fn() {}
}

mod unknown_lint_allow {
    #[cfg_attr(marker, allow(marker::some_unknown_lint_that_doesnt_exist))]
    fn foo() {}
}
