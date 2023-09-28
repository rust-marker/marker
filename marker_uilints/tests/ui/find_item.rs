macro_rules! ignore_macro_magic {
    () => {
        static FIND_ITEM: u32 = 4;
    };
}

mod find_real_item {
    static FIND_ITEM: u32 = 4;
}

mod ignore_macro_item {
    ignore_macro_magic!();
}

fn main() {}
