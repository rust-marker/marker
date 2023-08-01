#![feature(register_tool)]
#![register_tool(marker)]

const FIND_ME_DEFAULT: i32 = 0;

#[allow(marker::item_with_test_name)]
const FIND_ME_ALLOW: i32 = 0;

#[deny(marker::item_with_test_name)]
const FIND_ME_DENY: i32 = 0;
