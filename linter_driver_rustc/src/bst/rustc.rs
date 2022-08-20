use std::cell::{OnceCell, RefCell};

use linter_adapter::context::DriverContext;
use rustc_data_structures::fx::FxHashMap;
use rustc_lint::{LateContext, Level as RustcLevel, Lint as RustcLint, LintContext};

use linter_api::{
    ast::{
        Lifetime, Span, SpanOwner, SpanSource, Symbol,
    },
    context::AstContext,
    lint::{Level, Lint, MacroReport},
};
use rustc_span::BytePos;

use super::{api_span_from_rustc_span, item::rustc_item_id_from_api_item_id, RustcLifetime};

pub struct RustcContext<'ast, 'tcx> {
    pub(crate) ast_cx: OnceCell<&'ast AstContext<'ast>>,
    pub(crate) rustc_cx: &'ast LateContext<'tcx>,
    pub(crate) lint_map: RefCell<FxHashMap<&'ast Lint, &'static RustcLint>>,
    pub(crate) span_source_map: RefCell<FxHashMap<SpanSource<'ast>, (BytePos, rustc_span::hygiene::SyntaxContext)>>,
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

impl<'ast, 'tcx> DriverContext<'ast> for RustcContext<'ast, 'tcx> {
    fn emit_lint(&self, lint: &'static Lint, msg: &str, span: &Span<'ast>) {
        let (in_file_pos, rustc_ctxt) = self
            .span_source_map
            .borrow()
            .get(&span.source())
            .map(|&(p, c)| (p, c))
            .unwrap();
        let rustc_file = self.rustc_cx.sess().source_map().lookup_source_file(in_file_pos);
        #[expect(clippy::cast_possible_truncation)]
        let lo = BytePos(span.start() as u32) + rustc_file.start_pos;
        #[expect(clippy::cast_possible_truncation)]
        let hi = BytePos(span.end() as u32) + rustc_file.start_pos;
        let span = rustc_span::DUMMY_SP.with_ctxt(rustc_ctxt).with_lo(lo).with_hi(hi);

        let mut map = self.lint_map.borrow_mut();
        self.rustc_cx
            .struct_span_lint(to_leaked_rustc_lint(&mut *map, lint), span, |diag| {
                let mut diag = diag.build(msg);
                diag.emit();
            });
    }

    fn get_span(&'ast self, owner: &SpanOwner) -> &'ast Span<'ast> {
        match &owner {
            // FIXME use `api_span_from_rustc_span`
            SpanOwner::Item(api_id) => {
                let rustc_def_id = rustc_item_id_from_api_item_id(*api_id);
                let rustc_item_id = rustc_hir::ItemId {
                    def_id: rustc_hir::def_id::LocalDefId {
                        local_def_index: rustc_def_id.index,
                    },
                };
                let item = self.rustc_cx.tcx.hir().item(rustc_item_id);
                let api_span = self.alloc_with(|| api_span_from_rustc_span(self, item.span));

                let mut map = self.span_source_map.borrow_mut();
                map.entry(api_span.source())
                    .or_insert((item.span.lo(), item.span.data().ctxt));

                api_span
            }, //
            SpanOwner::Body(_) => todo!(),
            SpanOwner::SpecificSpan(_span_id) => todo!(),
        }
    }

    fn span_snippet(&self, span: &linter_api::ast::Span) -> Option<&'ast str> {
        match span.source() {
            linter_api::ast::SpanSource::File(path) => {
                let name = rustc_span::FileName::Real(rustc_span::RealFileName::LocalPath(path.clone()));
                let file = self.rustc_cx.sess().source_map().get_source_file(&name)?;
                let src = file
                    .as_ref()
                    .src
                    .as_ref()
                    .and_then(|src| src.get(span.start()..span.end()));
                Some(self.buffer.alloc_str(src?))
                // let text = file_src.get(span.start()..span.end())?;
                // let src = self.alloc_with(|| text.to_string());
                // Some(src.as)
            },
            linter_api::ast::SpanSource::Macro(_) => todo!(),
        }
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

    pub fn ast_cx(&self) -> &AstContext<'ast> {
        self.ast_cx
            .get()
            .expect("directly set after creation and should therefore be valid")
    }
}

impl<'ast, 'tcx> RustcContext<'ast, 'tcx> {
    #[must_use]
    #[allow(clippy::unused_self)]
    pub fn new_symbol(&'ast self, sym: rustc_span::symbol::Symbol) -> Symbol {
        Symbol::new(sym.as_u32())
    }

    pub fn new_lifetime(&'ast self) -> &'ast dyn Lifetime<'ast> {
        self.buffer.alloc_with(|| RustcLifetime {})
    }
}

impl<'ast, 'tcx> RustcContext<'ast, 'tcx> {
    #[must_use]
    pub fn new(ctx: &'ast LateContext<'tcx>, buffer: &'ast bumpalo::Bump) -> Self {
        Self {
            ast_cx: OnceCell::new(),
            rustc_cx: ctx,
            lint_map: RefCell::default(),
            span_source_map: RefCell::default(),
            buffer,
        }
    }
}
