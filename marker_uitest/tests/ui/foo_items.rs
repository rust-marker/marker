mod good {
    fn foo() {}

    static FOO: i32 = 0;

    struct Foo {
        foo: i32,
    }
}

mod foo {
    use crate::good as foo;

    const FOO: i32 = 0;

    enum Foo {
        Foo,
    }
}

fn main() {}
