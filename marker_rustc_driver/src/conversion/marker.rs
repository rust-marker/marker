//! This module and its sub modules form the translation layer from rustc's
//! internal representation to markers representation. All conversion methods
//! are implemented as methods of the [`MarkerConverterInner`] to group them
//! together and share access to common objects easily.

mod ast;
mod common;
mod sem;
mod span;

use std::cell::{OnceCell, RefCell};

use crate::context::storage::Storage;
use marker_api::{
    ast::{Body, CommonItemData, Crate, EnumVariant, ItemField, ModItem, Visibility as AstVisibility},
    common::{Level, SymbolId},
    prelude::*,
    sem::{Visibility as SemVisibility, VisibilityKind},
    span::{ExpnInfo, FilePos, Span, SpanSource},
};
use rustc_hash::FxHashMap;
use rustc_hir as hir;

/// An interface to convert rustc's IR to marker types.
///
/// This is a wrapper for [`MarkerConverterInner`] which is responsible for the
/// actual conversion. The conversion code from [`MarkerConverterInner`] has certain
/// expectations when it comes to the internal state. Using this wrapper ensures,
/// that these expectations are always fulfilled.
pub struct MarkerConverter<'ast, 'tcx> {
    inner: MarkerConverterInner<'ast, 'tcx>,
}

impl<'ast, 'tcx> MarkerConverter<'ast, 'tcx> {
    pub fn new(rustc_cx: rustc_middle::ty::TyCtxt<'tcx>, storage: &'ast Storage<'ast>) -> Self {
        Self {
            inner: MarkerConverterInner::new(rustc_cx, storage),
        }
    }

    fn with_body<F, R>(&self, hir_id: hir::HirId, with: F) -> R
    where
        F: FnOnce(&MarkerConverterInner<'ast, 'tcx>) -> R,
    {
        let map = self.inner.rustc_cx.hir();
        let owner = map.enclosing_body_owner(hir_id);
        let body_id = map.body_owned_by(owner);

        // When this is called, the body information should always be `None`
        // we therefore don't need to remember and reset it.
        let old_body = self.inner.rustc_body.replace(Some(body_id));
        debug_assert_eq!(old_body, None);
        self.inner.fill_rustc_ty_check();

        let res = with(&self.inner);
        self.inner.rustc_body.replace(None);
        self.inner.rustc_ty_check.replace(None);

        res
    }

    pub fn expr_ty(&self, id: hir::HirId) -> marker_api::sem::TyKind<'ast> {
        self.with_body(id, |inner| {
            let ty = inner.rustc_ty_check().node_type(id);
            inner.to_sem_ty(ty)
        })
    }

    forward_to_inner!(pub fn to_lint_level(&self, level: rustc_lint::Level) -> Level);

    pub fn body(&self, id: hir::BodyId) -> &'ast Body<'ast> {
        // Check the cache
        let api_id = self.inner.to_body_id(id);
        if let Some(body) = self.inner.bodies.borrow().get(&api_id) {
            return body;
        }
        let rustc_body = self.inner.rustc_cx.hir().body(id);
        self.inner.to_body(rustc_body)
    }

    pub fn item(&self, item_id: hir::ItemId) -> Option<ItemKind<'ast>> {
        // Check the cache
        let api_id = self.inner.to_item_id(item_id);
        if let Some(item) = self.inner.items.borrow().get(&api_id) {
            return Some(*item);
        }

        self.inner.to_item_from_id(item_id)
    }

    pub fn stmt(&self, hir_id: hir::HirId) -> Option<StmtKind<'ast>> {
        // Check the cache
        let id = self.inner.to_stmt_id(hir_id);
        if let Some(stmt) = self.inner.stmts.borrow().get(&id) {
            return Some(*stmt);
        }

        self.with_body(hir_id, |inner| {
            let Some(hir::Node::Stmt(stmt)) = inner.rustc_cx.hir().find(hir_id) else {
                return None;
            };
            inner.to_stmt(stmt)
        })
    }

    pub fn expr(&self, hir_id: hir::HirId) -> Option<ExprKind<'ast>> {
        // Check the cache
        let id = self.inner.to_expr_id(hir_id);
        if let Some(expr) = self.inner.exprs.borrow().get(&id) {
            return Some(*expr);
        }

        self.with_body(hir_id, |inner| {
            let Some(hir::Node::Expr(expr)) = inner.rustc_cx.hir().find(hir_id) else {
                return None;
            };
            Some(inner.to_expr(expr))
        })
    }

    pub fn variant(&self, id: VariantId) -> Option<&'ast EnumVariant<'ast>> {
        // Lint crates only gain access to ids of fields and variants, that are
        // in scope. Marker's conversion first transforms the entire crate. Any enums
        // defined on this level will therefore be available in the cache.
        //
        // Enums and Structs defined inside bodies, will add their fields and variants
        // to the cache, ensuring again, that they are accessible, if a user asks for them.
        //
        // This simply means, that it's enough to only check the cache. This is also
        // the reason why this function takes an API id and not the rustc variant.
        self.inner.variants.borrow().get(&id).copied()
    }

    pub fn field(&self, id: FieldId) -> Option<&'ast ItemField<'ast>> {
        // See docs of the `variant` method for an explanation, why it's
        // enough to only check the cache.
        self.inner.fields.borrow().get(&id).copied()
    }

    forward_to_inner!(pub fn to_ty_def_id(&self, id: hir::def_id::DefId) -> TyDefId);
    forward_to_inner!(pub fn to_span(&self, rustc_span: rustc_span::Span) -> Span<'ast>);
    forward_to_inner!(pub fn to_span_source(&self, rust_span: rustc_span::Span) -> SpanSource<'ast>);
    forward_to_inner!(pub fn try_to_expn_info(&self, expn_id: rustc_span::ExpnId) -> Option<&'ast ExpnInfo<'ast>>);
    forward_to_inner!(pub fn try_to_span_pos(
        &self,
        scx: rustc_span::SyntaxContext,
        pos: rustc_span::BytePos,
    ) -> Option<FilePos<'ast>>);
    forward_to_inner!(pub fn local_crate(
        &self,
    ) -> &'ast Crate<'ast>);
}

macro_rules! forward_to_inner {
    (pub fn $fn_name:ident(&self $(, $arg_name:ident: $arg_ty:ty)* $(,)?) -> $ret_ty:ty) => {
        pub fn $fn_name(&self $(, $arg_name: $arg_ty)*) -> $ret_ty {
            self.inner.$fn_name($($arg_name, )*)
        }
    };
}
use forward_to_inner;

struct MarkerConverterInner<'ast, 'tcx> {
    rustc_cx: rustc_middle::ty::TyCtxt<'tcx>,
    storage: &'ast Storage<'ast>,

    // Converted nodes cache
    krate: OnceCell<&'ast Crate<'ast>>,
    items: RefCell<FxHashMap<ItemId, ItemKind<'ast>>>,
    bodies: RefCell<FxHashMap<BodyId, &'ast Body<'ast>>>,
    exprs: RefCell<FxHashMap<ExprId, ExprKind<'ast>>>,
    stmts: RefCell<FxHashMap<StmtId, StmtKind<'ast>>>,
    fields: RefCell<FxHashMap<FieldId, &'ast ItemField<'ast>>>,
    variants: RefCell<FxHashMap<VariantId, &'ast EnumVariant<'ast>>>,

    // Cached/Dummy values
    builtin_span_source: &'ast marker_api::span::BuiltinInfo<'ast>,
    num_symbols: RefCell<FxHashMap<u32, SymbolId>>,

    /// Lang-items are weird, and if I'm being honest, I'm uncertain that I
    /// completely understand them. Anyways, here it goes, this is my current
    /// understanding:
    ///
    /// User written paths like `String` are always stored as `Resolved` or
    /// `TypeRelative`. The `QPath::LangItem` is used for code that is
    /// generated by the compiler, for instance from `format_args!()`. This
    /// allows the compiler to add paths, without the need to resolve them,
    /// since the ID of lang-items is known. However, this makes it more
    /// complicated for marker, since there is no real path to convert. Also,
    /// there seems to be no real way to transform a lang-item to the path of
    /// the target item. This means manual mapping in marker... FUN...
    ///
    /// This is a map, connecting the lang-items with the [`SymbolId`]s which
    /// would correspond to the path. This mapping doesn't cover all items, as
    /// some should never be mapped but converted to other representations.
    /// (Adding all items at once would also not be too much fun)
    /// Also, lang items are common enough, to have this map filled by default,
    /// almost every `format_args` call uses them.
    ///
    /// The map is filled in [`Self::fill_create_lang_item_map`].
    lang_item_map: RefCell<FxHashMap<hir::LangItem, SymbolId>>,

    // Context information
    /// This holds the [`hir::BodyId`] of the body that is currently being
    /// converted. This may be [`None`] for items, but should always be [`Some`]
    /// for expressions, since they can (AFAIK) only occur inside bodies.
    /// Individual expressions can be requested via the driver context, however,
    /// this driver only provides IDs of converted expressions, meaning that
    /// the requested expression would be returned from cache and not
    /// require additional translations.
    rustc_body: RefCell<Option<hir::BodyId>>,
    /// Requested on demand from rustc using a [`hir::BodyId`] see
    /// [`MarkerConverterInner::rustc_body`] for more information
    rustc_ty_check: RefCell<Option<&'tcx rustc_middle::ty::TypeckResults<'tcx>>>,
}

// General util functions
impl<'ast, 'tcx> MarkerConverterInner<'ast, 'tcx> {
    fn new(rustc_cx: rustc_middle::ty::TyCtxt<'tcx>, storage: &'ast Storage<'ast>) -> Self {
        let s = Self {
            rustc_cx,
            storage,
            krate: OnceCell::default(),
            items: RefCell::default(),
            bodies: RefCell::default(),
            exprs: RefCell::default(),
            stmts: RefCell::default(),
            fields: RefCell::default(),
            variants: RefCell::default(),
            builtin_span_source: storage.alloc(marker_api::span::BuiltinInfo::default()),
            num_symbols: RefCell::default(),
            lang_item_map: RefCell::default(),
            rustc_body: RefCell::default(),
            rustc_ty_check: RefCell::default(),
        };

        s.fill_create_lang_item_map();

        s
    }

    /// See [`Self::lang_item_map`] for more context.
    fn fill_create_lang_item_map(&self) {
        use rustc_span::symbol::Symbol;
        #[rustfmt::skip]
        let list = [
            (hir::LangItem::TryTraitBranch, self.to_symbol_id(Symbol::intern("Try::branch"))),
            (hir::LangItem::FormatArgument, self.to_symbol_id(Symbol::intern("rustc_ast::format::FormatArgument"))),
            (hir::LangItem::FormatArguments, self.to_symbol_id(Symbol::intern("rustc_ast::format::FormatArguments"))),
            (hir::LangItem::FormatPlaceholder, self.to_symbol_id(Symbol::intern("rustc_ast::format::FormatPlaceholder"))),
            (hir::LangItem::FormatAlignment, self.to_symbol_id(Symbol::intern("rustc_ast::format::FormatAlignment"))),
            (hir::LangItem::FormatCount, self.to_symbol_id(Symbol::intern("rustc_ast::format::FormatCount"))),
            (hir::LangItem::FormatUnsafeArg, self.to_symbol_id(Symbol::intern("core::fmt::rt::UnsafeArg"))),
            (hir::LangItem::ResumeTy, self.to_symbol_id(Symbol::intern("lang_item::ResumeTy"))),
        ];

        self.lang_item_map.borrow_mut().extend(list);
    }

    pub fn fill_rustc_ty_check(&self) {
        let id = self
            .rustc_body
            .borrow()
            .expect("ty check can only be filled inside bodies");
        self.rustc_ty_check.replace(Some(self.rustc_cx.typeck_body(id)));
    }

    pub fn rustc_ty_check(&self) -> &rustc_middle::ty::TypeckResults<'tcx> {
        self.rustc_ty_check
            .borrow()
            .expect("MarkerConverterInner.rustc_ty_check is unexpectedly empty")
    }

    #[must_use]
    fn alloc<T>(&self, t: T) -> &'ast T {
        self.storage.alloc(t)
    }

    #[must_use]
    fn alloc_slice<T, I>(&self, iter: I) -> &'ast [T]
    where
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        self.storage.alloc_slice(iter)
    }

    pub fn with_body<U, F>(&self, rustc_body_id: hir::BodyId, f: F) -> U
    where
        F: FnOnce() -> U,
    {
        // Body-Translation-Stack push
        let prev_rustc_body_id = self.rustc_body.replace(Some(rustc_body_id));
        let prev_rustc_ty_check = self.rustc_ty_check.take();
        self.fill_rustc_ty_check();

        // Operation
        let res = f();

        // Body-Translation-Stack pop
        self.rustc_body.replace(prev_rustc_body_id);
        self.rustc_ty_check.replace(prev_rustc_ty_check);

        // Return result
        res
    }
}

impl<'ast, 'tcx> MarkerConverterInner<'ast, 'tcx> {
    #[must_use]
    fn local_crate(&self) -> &'ast Crate<'ast> {
        self.krate.get_or_init(|| {
            let krate = self.alloc(
                Crate::builder()
                    .id(self.to_crate_id(hir::def_id::LOCAL_CRATE))
                    .root_mod(self.local_crate_mod())
                    .build(),
            );

            let root_mod = krate.root_mod();
            self.items.borrow_mut().insert(root_mod.id(), ItemKind::Mod(root_mod));
            krate
        })
    }

    fn local_crate_mod(&self) -> ModItem<'ast> {
        let id = self.to_item_id(hir::def_id::DefId::from(hir::CRATE_OWNER_ID));
        let krate_mod = self.rustc_cx.hir().root_module();
        let ident = Ident::new(
            self.to_symbol_id(self.rustc_cx.crate_name(hir::def_id::LOCAL_CRATE)),
            self.to_span_id(rustc_span::DUMMY_SP),
        );
        let data = CommonItemData::builder()
            .id(id)
            .span(self.to_span_id(krate_mod.spans.inner_span))
            .vis(
                AstVisibility::builder()
                    .span(None)
                    .sem(SemVisibility::builder().kind(VisibilityKind::DefaultPub).build())
                    .build(),
            )
            .ident(ident)
            .build();
        ModItem::builder()
            .data(data)
            .items(self.to_items(krate_mod.item_ids))
            .build()
    }
}
