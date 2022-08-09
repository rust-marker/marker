use linter_api::{ast::Span, context::DriverCallbacks, lint::Lint};

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
        let wrapper_ptr: *const Self = self;
        DriverCallbacks {
            driver_context: wrapper_ptr.cast::<()>(),
            emit_lint,
        }
    }
}

#[expect(improper_ctypes_definitions)]
extern "C" fn emit_lint<'ast>(data: *const (), lint: &'static Lint, msg: &str, span: &dyn Span<'ast>) {
    let data = data.cast::<DriverContextWrapper<'ast>>();
    let wrapper: &'ast DriverContextWrapper<'ast> = unsafe { data.as_ref() }.unwrap();

    wrapper.driver_cx.emit_lint(lint, msg, span);
}

pub trait DriverContext<'ast> {
    fn emit_lint(&self, lint: &'static Lint, msg: &str, span: &dyn Span<'ast>);
}