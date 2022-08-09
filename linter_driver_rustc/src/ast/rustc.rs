use std::cell::RefCell;

use rustc_data_structures::fx::FxHashMap;
use rustc_lint::{LateContext, Level as RustcLevel, Lint as RustcLint, LintContext};

use linter_api::{
    ast::{
        ty::{Ty, TyKind},
        Ident, Lifetime, Span, Symbol,
    },
    lint::{Level, Lint, MacroReport},
};

use super::{ty::RustcTy, RustcLifetime, RustcSpan};

pub struct RustcContext<'ast, 'tcx> {
    pub(crate) rustc_cx: &'ast LateContext<'tcx>,
    pub(crate) lint_map: RefCell<FxHashMap<&'ast Lint, &'static RustcLint>>,
    /// All items should be created using the `alloc_*` functions. This ensures
    /// that we can later change the way we allocate and manage our memory
    buffer: &'ast bumpalo::Bump,
}

impl<'ast, 'tcx> std::fmt::Debug for RustcContext<'ast, 'tcx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RustcContext").finish()
    }
}

fn to_leaked_rustc_lint<'ast>(
    map: &mut FxHashMap<&'ast Lint, &'static RustcLint>,
    lint: &'ast Lint,
) -> &'static RustcLint {
    map.entry(lint).or_insert_with(|| {
        Box::leak(Box::new(RustcLint {
            name: lint.name,
            default_level: match lint.default_level {
                Level::Allow => RustcLevel::Allow,
                Level::Warn => RustcLevel::Warn,
                Level::Deny => RustcLevel::Deny,
                Level::Forbid => RustcLevel::Forbid,
                _ => unreachable!("added variant to lint::Level"),
            },
            desc: lint.explaination,
            edition_lint_opts: None,
            report_in_external_macro: match lint.report_in_macro {
                MacroReport::No | MacroReport::Local => false,
                MacroReport::All => true,
                _ => unreachable!("added variant to lint::MacroReport"),
            },
            future_incompatible: None,
            is_plugin: true,
            feature_gate: None,
            crate_level_only: false,
        }))
    })
}

pub extern "C" fn context_emit_lint<'ast, 'tcx: 'ast>(
    data: *const (),
    lint: &'static Lint,
    msg: &str,
    span: &dyn Span<'ast>,
) {
    let cx = data as *const RustcContext<'ast, 'tcx>;
    unsafe { cx.as_ref() }.unwrap().emit_lint_span(lint, msg, span);
}
pub extern "C" fn context_emit_lint_without_span<'ast, 'tcx: 'ast>(data: *const (), lint: &'static Lint, msg: &str) {
    let cx = data as *const RustcContext<'ast, 'tcx>;
    unsafe { cx.as_ref() }.unwrap().emit_lint(lint, msg);
}

impl<'ast, 'tcx> RustcContext<'ast, 'tcx> {
    fn emit_lint(&self, lint: &'ast Lint, s: &str) {
        let mut map = self.lint_map.borrow_mut();
        self.rustc_cx.lint(to_leaked_rustc_lint(&mut *map, lint), |diag| {
            let mut diag = diag.build(s);
            diag.emit();
        });
    }

    fn emit_lint_span(&self, lint: &'ast Lint, s: &str, sp: &dyn Span<'_>) {
        // Safety:
        //
        // Clearly this is probably not ideal but I did find this (answered by people much more
        // knowledgeable about lifetime/unsafe/transmute)
        // https://users.rust-lang.org/t/solved-transmute-between-trait-objects/13995/6
        //
        // In regard to the leak/forget, since we can keep this within the `'tcx` - `'ast` lifetime I don't
        // think we have to. We aren't returning a `dyn Trait + 'static` like if we were actually using the
        // `dyn Any` or the `dyn Span<'_>`. We can't use the `Any::downcast_ref` machinery here since we
        // have a non `'static` trait object in `Span` (at least I couldn't get it to work)
        #[allow(clippy::ptr_as_ptr, clippy::cast_ptr_alignment)]
        let down_span: &RustcSpan<'ast, 'tcx> = unsafe {
            let sp_ptr = sp as *const _ as *const (*mut (), *mut ());
            &*(sp_ptr as *const dyn std::any::Any as *const _)
        };

        let mut map = self.lint_map.borrow_mut();
        self.rustc_cx
            .struct_span_lint(to_leaked_rustc_lint(&mut *map, lint), down_span.span, |diag| {
                let mut diag = diag.build(s);
                diag.emit();
            });
    }
}

impl<'ast, 'tcx> RustcContext<'ast, 'tcx> {
    pub fn alloc_with<F, T>(&self, f: F) -> &'ast T
    where
        F: FnOnce() -> T,
    {
        self.buffer.alloc_with(f)
    }

    #[must_use]
    pub fn alloc_slice_from_iter<T, I>(&self, iter: I) -> &'ast [T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        self.buffer.alloc_slice_fill_iter(iter)
    }

    #[must_use]
    pub fn alloc_slice<T, F>(&self, len: usize, f: F) -> &'ast [T]
    where
        F: FnMut(usize) -> T,
    {
        self.buffer.alloc_slice_fill_with(len, f)
    }
}

impl<'ast, 'tcx> RustcContext<'ast, 'tcx> {
    #[must_use]
    pub fn new_span(&'ast self, span: rustc_span::Span) -> &'ast dyn Span<'ast> {
        self.buffer.alloc_with(|| RustcSpan::new(span, self))
    }

    #[must_use]
    #[allow(clippy::unused_self)]
    pub fn new_symbol(&'ast self, sym: rustc_span::symbol::Symbol) -> Symbol {
        Symbol::new(sym.as_u32())
    }

    #[must_use]
    pub fn new_ident(&'ast self, ident: rustc_span::symbol::Ident) -> &'ast Ident<'ast> {
        self.buffer
            .alloc_with(|| Ident::new(self.new_symbol(ident.name), self.new_span(ident.span)))
    }

    #[must_use]
    pub fn new_ty(&'ast self, kind: TyKind<'ast>, is_infered: bool) -> &'ast dyn Ty<'ast> {
        self.buffer.alloc_with(|| RustcTy::new(self, kind, is_infered))
    }

    pub fn new_lifetime(&'ast self) -> &'ast dyn Lifetime<'ast> {
        self.buffer.alloc_with(|| RustcLifetime {})
    }
}

impl<'ast, 'tcx> RustcContext<'ast, 'tcx> {
    #[must_use]
    pub fn new(ctx: &'ast LateContext<'tcx>, buffer: &'ast bumpalo::Bump) -> Self {
        Self {
            rustc_cx: ctx,
            lint_map: RefCell::default(),
            buffer,
        }
    }
}
