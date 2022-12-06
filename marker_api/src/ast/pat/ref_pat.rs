use super::CommonPatData;

#[repr(C)]
#[derive(Debug)]
pub struct RefPat<'ast> {
    data: CommonPatData<'ast>,
}

super::impl_pat_data!(RefPat<'ast>, Ref);

#[cfg(feature = "driver-api")]
impl<'ast> RefPat<'ast> {
    pub fn new(data: CommonPatData<'ast>) -> Self {
        Self { data }
    }
}
