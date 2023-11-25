use super::CommonTyData;

/// The placeholder type, signalling that the semantic type is still unstable
/// and therefor not represented as part of the API.
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct UnstableTy<'ast> {
    data: CommonTyData<'ast>,
}

super::impl_ty_data!(UnstableTy<'ast>, Unstable);
