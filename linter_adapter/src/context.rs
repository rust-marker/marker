use linter_api::{
    ast::{Span, SpanOwner},
    context::DriverCallbacks,
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
            driver_context:  unsafe { &*(self as *const DriverContextWrapper).cast::<()>() },
            emit_lint,
            get_span,
            span_snippet,
        }
    }
}

#[expect(improper_ctypes_definitions)]
extern "C" fn emit_lint<'ast>(data: &(), lint: &'static Lint, msg: &str, span: &Span<'ast>) {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.emit_lint(lint, msg, span);
}

extern "C" fn get_span<'ast>(data: &(), owner: &SpanOwner) -> &'ast Span<'ast> {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.get_span(owner)
}

#[expect(improper_ctypes_definitions)]
extern "C" fn span_snippet<'ast>(data: &(), span: &Span) -> Option<&'ast str> {
    let wrapper = unsafe { &*(data as *const ()).cast::<DriverContextWrapper>() };
    wrapper.driver_cx.span_snippet(span)
}

pub trait DriverContext<'ast> {
    fn emit_lint(&self, lint: &'static Lint, msg: &str, span: &Span<'ast>);
    fn get_span(&'ast self, owner: &SpanOwner) -> &'ast Span<'ast>;
    fn span_snippet(&self, span: &Span) -> Option<&'ast str>;
}
