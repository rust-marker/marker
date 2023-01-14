use super::CommonPatData;

#[repr(C)]
#[derive(Debug)]
pub struct UnstablePat<'ast> {
    data: CommonPatData<'ast>,
}

super::impl_pat_data!(UnstablePat<'ast>, Unstable);

#[cfg(feature = "driver-api")]
impl<'ast> UnstablePat<'ast> {
    pub fn new(data: CommonPatData<'ast>) -> Self {
        Self { data }
    }
}
