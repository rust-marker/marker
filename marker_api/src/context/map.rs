use crate::{
    ast::{Body, EnumVariant, ExprKind, ItemField, ItemKind, StmtKind},
    common::{BodyId, ExprId, FieldId, ItemId, Level, StmtId, VariantId},
    ffi,
    lint::Lint,
    prelude::{HasNodeId, NodeId},
};

/// A map, which allows the request of AST nodes by their ids. An instance of this
/// map can be accessed from [`MarkerContext::ast`](super::MarkerContext::ast).
///
/// # Availability of Nodes
///
/// Rustc, as a compiler and driver for Marker, compiles each crate individually
/// [^compilation-unit]. This setup means that at a given time, there are only
/// the AST of one crate available. Therefore, it can happen that the AST node
/// representation for a valid ID is unavailable.
///
/// Generally speaking, it's advised to use the failable variant of the provided
/// functions and just bail if the expected node is unavailable. The API also
/// provides some functions that automatically unwrap the requested nodes. These
/// functions, starting with `unwrap_`, are great for prototyping but should be
/// used carefully.
///
/// # Future plans
///
/// While the driver might not have the AST of a dependency loaded, it
/// usually has some semantic information about what types and functions are
/// available. Marker should provide some way to request this semantic information
/// based on the ID. (See: <https://github.com/rust-marker/marker/issues/266>)
///
/// [^compilation-unit]: For more context, ASTs can take up a lot of space.
///     Splitting the compilation of a project into separate compilation units
///     is one way to handle memory better. This approach also allows for some
///     optimizations. For example, Rustc was able to reduce the size of some
///     structs, as it was known that they only need to handle one crate at a time.
#[repr(C)]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
pub struct AstMap<'ast> {
    callbacks: AstMapCallbacks<'ast>,
}

impl<'ast> AstMap<'ast> {
    pub fn lint_level_at(&self, lint: &'static Lint, node: impl HasNodeId) -> Level {
        (self.callbacks.lint_level_at)(self.callbacks.data, lint, node.node_id())
    }

    /// Returns the [`ItemKind`] belonging to the given [`ItemId`], if available.
    ///
    /// Checkout the documentation of [`AstMap`] for more information, when a node
    /// might be unavailable, even if the given ID is valid.
    pub fn item(&self, id: ItemId) -> Option<ItemKind<'ast>> {
        (self.callbacks.item)(self.callbacks.data, id).copy()
    }

    /// Returns the [`ItemKind`] belonging to the given [`ItemId`].
    ///
    /// # Panics
    ///
    /// Panics if the requested item is currently unavailable. Checkout the
    /// documentation of [`AstMap`] for an explanation, when AST nodes might be
    /// unavailable. [`AstMap::item`] can be used as a non-panicking alternative.
    pub fn unwrap_item(&self, id: ItemId) -> ItemKind<'ast> {
        self.item(id)
            .unwrap_or_else(|| panic!("The requested item is unavailable (id = {id:?})"))
    }

    /// Returns the [`EnumVariant`] belonging to the given [`VariantId`], if available.
    ///
    /// Checkout the documentation of [`AstMap`] for more information, when a node
    /// might be unavailable, even if the given ID is valid.
    pub fn variant(&self, id: VariantId) -> Option<&EnumVariant<'ast>> {
        (self.callbacks.variant)(self.callbacks.data, id).copy()
    }

    /// Returns the [`EnumVariant`] belonging to the given [`VariantId`].
    ///
    /// # Panics
    ///
    /// Panics if the requested enum variant is currently unavailable. Checkout the
    /// documentation of [`AstMap`] for an explanation, when AST nodes might be
    /// unavailable. [`AstMap::variant`] can be used as a non-panicking alternative.
    pub fn unwrap_variant(&self, id: VariantId) -> &EnumVariant<'ast> {
        self.variant(id)
            .unwrap_or_else(|| panic!("The requested enum variant is unavailable (id = {id:?})"))
    }

    /// Returns the [`ItemField`] belonging to the given [`FieldId`], if available.
    ///
    /// Checkout the documentation of [`AstMap`] for more information, when a node
    /// might be unavailable, even if the given ID is valid.
    pub fn field(&self, id: FieldId) -> Option<&ItemField<'ast>> {
        (self.callbacks.field)(self.callbacks.data, id).copy()
    }

    /// Returns the [`ItemField`] belonging to the given [`FieldId`].
    ///
    /// # Panics
    ///
    /// Panics if the requested enum variant is currently unavailable. Checkout the
    /// documentation of [`AstMap`] for an explanation, when AST nodes might be
    /// unavailable. [`AstMap::variant`] can be used as a non-panicking alternative.
    pub fn unwrap_field(&self, id: FieldId) -> &ItemField<'ast> {
        self.field(id)
            .unwrap_or_else(|| panic!("The requested field is unavailable (id = {id:?})"))
    }

    /// Returns the [`Body`] belonging to the given [`BodyId`].
    pub fn body(&self, id: BodyId) -> &Body<'ast> {
        (self.callbacks.body)(self.callbacks.data, id)
    }

    /// Returns the [`StmtKind`] belonging to the given [`StmtId`].
    pub fn stmt(&self, id: StmtId) -> StmtKind<'ast> {
        (self.callbacks.stmt)(self.callbacks.data, id)
    }

    /// Returns the [`ExprKind`] belonging to the given [`ExprId`].
    pub fn expr(&self, id: ExprId) -> ExprKind<'ast> {
        (self.callbacks.expr)(self.callbacks.data, id)
    }
}

#[repr(C)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
#[cfg_attr(feature = "driver-api", derive(typed_builder::TypedBuilder))]
struct AstMapCallbacks<'ast> {
    /// The data that will be used as the first argument for the callback functions.
    /// The content of this data is defined by the driver (or by marker_adapter on behalf
    /// of the driver)
    pub data: &'ast AstMapData,

    pub item: extern "C" fn(data: &'ast AstMapData, id: ItemId) -> ffi::FfiOption<ItemKind<'ast>>,
    pub variant: extern "C" fn(data: &'ast AstMapData, id: VariantId) -> ffi::FfiOption<&'ast EnumVariant<'ast>>,
    pub field: extern "C" fn(data: &'ast AstMapData, id: FieldId) -> ffi::FfiOption<&'ast ItemField<'ast>>,
    pub body: extern "C" fn(data: &'ast AstMapData, id: BodyId) -> &'ast Body<'ast>,
    pub stmt: extern "C" fn(data: &'ast AstMapData, id: StmtId) -> StmtKind<'ast>,
    pub expr: extern "C" fn(data: &'ast AstMapData, id: ExprId) -> ExprKind<'ast>,

    pub lint_level_at: extern "C" fn(data: &'ast AstMapData, lint: &'static Lint, node: NodeId) -> Level,
}

/// This type is used by [`AstMapCallbacks`] as the first argument to every
/// function. For more information, see the documentation of the `data` field
/// or from `marker_adapter::context`.
///
/// This type should never be constructed and is only meant as a pointer
/// casting target.
#[repr(C)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
struct AstMapData {
    /// `#[repr(C)]` requires a field, to make this a proper type. Using usize
    /// ensures that the structs has the same alignment requirement as a pointer.
    _data: usize,
}
