use super::CommonPatData;

#[repr(C)]
#[derive(Debug)]
pub struct RestPat<'ast> {
    data: CommonPatData<'ast>,
}

super::impl_pat_data!(RestPat<'ast>, Rest);

#[cfg(feature = "driver-api")]
impl<'ast> RestPat<'ast> {
    pub fn new(data: CommonPatData<'ast>) -> Self {
        Self { data }
    }
}
