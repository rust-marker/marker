use linter_api::{
    ast::{item::ItemKind, ItemId, Span, SpanOwner, SymbolId},
    context::DriverCallbacks,
    ffi::{self, FfiOption},
    lint::Lint,
};

/// ### Safety
///
/// `&dyn` objects are theoretically not FFI safe since their type layout can
/// change and calling functions on them would require a stable ABI which Rust
/// doesn't provide.
///
/// In this case, the `DriverContextWrapper` will be passes as a `*const ()`
/// pointer to [`DriverCallbacks`] which will do nothing with this data other
/// than giving it back to functions declared in this module. Since the `&dyn`
/// object is created, only used here and everything is compiled during the same
/// compiler run, it should be safe to use `&dyn`.
#[repr(C)]
pub struct DriverContextWrapper<'ast> {
    driver_cx: &'ast dyn DriverContext<'ast>,
}

impl<'ast> DriverContextWrapper<'ast> {
    #[must_use]
    pub fn new(driver_cx: &'ast dyn DriverContext<'ast>) -> Self {
        Self { driver_cx }
    }

    #[must_use]
    pub fn create_driver_callback(&'ast self) -> DriverCallbacks<'ast> {
        DriverCallbacks {
            driver_context: unsafe { &*(self as *const DriverContextWrapper).cast::<()>() },
            emit_lint,
            item,
            get_span,
            span_snippet,
            symbol_str,
        }
    }
}

#[allow(improper_ctypes_definitions, reason = "fp because `ItemKind` is non-exhaustive")]
extern "C" fn item<'ast>(data: &(), id: ItemId) -> FfiOption<ItemKind<'ast>> {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.item(id).into()
}

extern "C" fn emit_lint<'ast>(data: &(), lint: &'static Lint, msg: ffi::Str, span: &Span<'ast>) {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.emit_lint(lint, (&msg).into(), span);
}

extern "C" fn get_span<'ast>(data: &(), owner: &SpanOwner) -> &'ast Span<'ast> {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.get_span(owner)
}

extern "C" fn span_snippet<'ast>(data: &(), span: &Span) -> ffi::FfiOption<ffi::Str<'ast>> {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.span_snippet(span).map(Into::into).into()
}

extern "C" fn symbol_str<'ast>(data: &(), sym: SymbolId) -> ffi::Str<'ast> {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.symbol_str(sym).into()
}

pub trait DriverContext<'ast> {
    fn item(&'ast self, id: ItemId) -> Option<ItemKind<'ast>>;
    fn emit_lint(&'ast self, lint: &'static Lint, msg: &str, span: &Span<'ast>);
    fn get_span(&'ast self, owner: &SpanOwner) -> &'ast Span<'ast>;
    fn span_snippet(&'ast self, span: &Span) -> Option<&'ast str>;
    fn symbol_str(&'ast self, sym: SymbolId) -> &'ast str;
}
