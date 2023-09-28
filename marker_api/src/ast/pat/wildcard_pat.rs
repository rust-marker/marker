use super::CommonPatData;

#[repr(C)]
#[derive(Debug)]
pub struct WildcardPat<'ast> {
    data: CommonPatData<'ast>,
}

super::impl_pat_data!(WildcardPat<'ast>, Wildcard);

#[cfg(feature = "driver-api")]
impl<'ast> WildcardPat<'ast> {
    pub fn new(data: CommonPatData<'ast>) -> Self {
        Self { data }
    }
}
