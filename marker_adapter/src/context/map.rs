use marker_api::{
    ast::{EnumVariant, ItemField},
    context::{AstMap, AstMapCallbacks, AstMapData},
    ffi,
    lint::Level,
    prelude::*,
};

#[repr(C)]
pub struct AstMapWrapper<'ast> {
    driver: &'ast dyn AstMapDriver<'ast>,
}

impl<'ast> AstMapWrapper<'ast> {
    pub fn new(driver: &'ast dyn AstMapDriver<'ast>) -> Self {
        Self { driver }
    }

    #[must_use]
    pub fn create_callbacks(&'ast self) -> AstMap<'ast> {
        AstMap::builder()
            .callbacks(AstMapCallbacks {
                data: unsafe { &*(self as *const AstMapWrapper).cast::<AstMapData>() },
                item,
                variant,
                field,
                body,
                stmt,
                expr,
                lint_level_at,
            })
            .build()
    }
}

/// The driver trait for [`AstMap`](marker_api::context::AstMap).
pub trait AstMapDriver<'ast> {
    fn item(&'ast self, id: ItemId) -> Option<ItemKind<'ast>>;
    fn variant(&'ast self, id: VariantId) -> Option<&'ast EnumVariant<'ast>>;
    fn field(&'ast self, id: FieldId) -> Option<&'ast ItemField<'ast>>;
    fn body(&'ast self, id: BodyId) -> &'ast ast::Body<'ast>;
    fn stmt(&'ast self, id: StmtId) -> StmtKind<'ast>;
    fn expr(&'ast self, id: ExprId) -> ExprKind<'ast>;

    fn lint_level_at(&'ast self, lint: &'static Lint, node: NodeId) -> Level;
}

#[allow(improper_ctypes_definitions)] // FP because `ItemKind` is non-exhaustive
extern "C" fn item<'ast>(data: &'ast AstMapData, id: ItemId) -> ffi::FfiOption<ItemKind<'ast>> {
    unsafe { as_driver(data) }.item(id).into()
}
extern "C" fn variant<'ast>(data: &'ast AstMapData, id: VariantId) -> ffi::FfiOption<&'ast EnumVariant<'ast>> {
    unsafe { as_driver(data) }.variant(id).into()
}
extern "C" fn field<'ast>(data: &'ast AstMapData, id: FieldId) -> ffi::FfiOption<&'ast ItemField<'ast>> {
    unsafe { as_driver(data) }.field(id).into()
}
extern "C" fn body<'ast>(data: &'ast AstMapData, id: BodyId) -> &'ast ast::Body<'ast> {
    unsafe { as_driver(data) }.body(id)
}
#[allow(improper_ctypes_definitions)] // FP because `StmtKind` is non-exhaustive
extern "C" fn stmt<'ast>(data: &'ast AstMapData, id: StmtId) -> StmtKind<'ast> {
    unsafe { as_driver(data) }.stmt(id)
}
#[allow(improper_ctypes_definitions)] // FP because `ExprKind` is non-exhaustive
extern "C" fn expr<'ast>(data: &'ast AstMapData, id: ExprId) -> ExprKind<'ast> {
    unsafe { as_driver(data) }.expr(id)
}

#[allow(improper_ctypes_definitions)] // FP because `NodeId` is non-exhaustive
extern "C" fn lint_level_at<'ast>(data: &'ast AstMapData, lint: &'static Lint, node: NodeId) -> Level {
    unsafe { as_driver(data) }.lint_level_at(lint, node)
}

/// # Safety
/// `data` must be a valid pointer to [`AstMapDriver`]
unsafe fn as_driver<'ast>(data: &'ast AstMapData) -> &'ast dyn AstMapDriver<'ast> {
    let wrapper = &*(data as *const AstMapData).cast::<AstMapWrapper>();
    wrapper.driver
}
