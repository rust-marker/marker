use expect_test::Expect;

#[track_caller]
pub(crate) fn assert_size_of<T>(expected: &Expect) {
    let actual = std::mem::size_of::<T>();
    expected.assert_eq(&actual.to_string());
}
