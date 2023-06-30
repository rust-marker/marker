mod find_me {
    const FIND_ME_CONST: i32 = 0;

    static FIND_ME_STATIC: i32 = 0;

    pub fn find_me_fn() {}

    struct FindMeStruct {
        find_me_field: i32,
    }
    enum FindMeEnum {
        FindMe,
    }
    union FindMeUnion {
        a: i32,
        b: u32,
    }
    trait FindMeTrait {}
}

use find_me::find_me_fn as find_me;

fn main() {}
