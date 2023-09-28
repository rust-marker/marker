use std::ops::ControlFlow;

use marker_api::{
    ast::{
        item::{EnumVariant, Field},
        BodyId,
    },
    prelude::*,
};

/// This enum defines the scope that a given [`Visitor`] should traverse.
///
/// By default, it's recommended to not visit any nested bodies. Most analysis operates
/// on the item or expression level. Variables and control flow statements are only
/// effective in the current body. Visiting nested bodies, can add a bunch of noise and
/// cause confusing results. Here is an example:
///
/// ```
/// fn foo() {
///     let x = 0;
///     let a = 1;
///
///     // A function inside `foo()` with it's own body. This one will only
///     // be visited if `VisitorScope::AllBodies` is specified.
///     fn bar(x: u32) {
///         // The printed `x` is different from the `x` of `foo()`
///         // since it comes from the function parameter of `bar()`
///         println!("The magic number is {x}");
///         // The `return` statement only affects the `bar` function.
///         // `foo()` will just continue executing, when this is called
///         return;
///     }
///
///     bar(a);
/// }
/// ```
///
/// The target scope is checked when the respective `traverse_*` function is called
/// For example, [`traverse_body`] will visit a given body [`Body`], but will not enter
/// nested bodies, unless [`AllBodies`](VisitorScope::AllBodies) is defined.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Default)]
pub enum VisitorScope {
    /// All bodies are visited, this includes bodies from nested items and closures.
    ///
    /// Only use this, if you're sure that you need the context of nested bodies, as the
    /// traversal of everything, can be expensive in comparison to the
    /// [`NoBodies`](VisitorScope::NoBodies) scope.
    AllBodies,
    /// This visits every node, in the current scope, but won't enter nested bodies.
    ///
    /// This is a good default, if you only want to analyze the context of the current
    /// item or expression.
    #[default]
    NoBodies,
}

pub trait Visitor<B> {
    /// Defines the [`scope`](VisitorScope) this visitor should use.
    ///
    /// This should return a constant value. The `traverse_*` functions might only
    /// check the scope once and cache the result.
    fn scope(&self) -> VisitorScope {
        VisitorScope::NoBodies
    }

    fn visit_item<'ast>(&mut self, _cx: &'ast MarkerContext<'ast>, _item: ItemKind<'ast>) -> ControlFlow<B> {
        ControlFlow::Continue(())
    }

    fn visit_field<'ast>(&mut self, _cx: &'ast MarkerContext<'ast>, _field: &'ast Field<'ast>) -> ControlFlow<B> {
        ControlFlow::Continue(())
    }

    fn visit_variant<'ast>(
        &mut self,
        _cx: &'ast MarkerContext<'ast>,
        _variant: &'ast EnumVariant<'ast>,
    ) -> ControlFlow<B> {
        ControlFlow::Continue(())
    }

    fn visit_body<'ast>(&mut self, _cx: &'ast MarkerContext<'ast>, _body: &'ast Body<'ast>) -> ControlFlow<B> {
        ControlFlow::Continue(())
    }

    fn visit_stmt<'ast>(&mut self, _cx: &'ast MarkerContext<'ast>, _stmt: StmtKind<'ast>) -> ControlFlow<B> {
        ControlFlow::Continue(())
    }

    fn visit_expr<'ast>(&mut self, _cx: &'ast MarkerContext<'ast>, _expr: ExprKind<'ast>) -> ControlFlow<B> {
        ControlFlow::Continue(())
    }
}

pub fn traverse_item<'ast, B>(
    cx: &'ast MarkerContext<'ast>,
    visitor: &mut dyn Visitor<B>,
    kind: ItemKind<'ast>,
) -> ControlFlow<B> {
    /// A small wrapper around [`traverse_body`] that checks the defined scope
    /// and validity of the [`BodyId`]
    fn traverse_body_id<'ast, B>(
        cx: &'ast MarkerContext<'ast>,
        visitor: &mut dyn Visitor<B>,
        id: Option<BodyId>,
    ) -> ControlFlow<B> {
        // Requesting the body from the API might be expensive. This check
        // prevents the requests if the body will not be used.
        if let VisitorScope::NoBodies = visitor.scope() {
            return ControlFlow::Continue(());
        }

        if let Some(body_id) = id {
            traverse_body(cx, visitor, cx.body(body_id))?;
        }

        ControlFlow::Continue(())
    }

    visitor.visit_item(cx, kind)?;

    match kind {
        ItemKind::Mod(module) => {
            for mod_item in module.items() {
                traverse_item(cx, visitor, *mod_item)?;
            }
        },
        ItemKind::Static(item) => {
            traverse_body_id(cx, visitor, item.body_id())?;
        },
        ItemKind::Const(item) => {
            traverse_body_id(cx, visitor, item.body_id())?;
        },
        ItemKind::Fn(item) => {
            traverse_body_id(cx, visitor, item.body_id())?;
        },
        ItemKind::Struct(item) => {
            for field in item.fields() {
                visitor.visit_field(cx, field)?;
            }
        },
        ItemKind::Union(item) => {
            for field in item.fields() {
                visitor.visit_field(cx, field)?;
            }
        },
        ItemKind::Enum(item) => {
            for variant in item.variants() {
                visitor.visit_variant(cx, variant)?;
                if let Some(const_expr) = variant.discriminant() {
                    traverse_expr(cx, visitor, const_expr.expr())?;
                }
            }
        },
        ItemKind::Trait(item) => {
            for assoc_item in item.items() {
                traverse_item(cx, visitor, assoc_item.as_item())?;
            }
        },
        ItemKind::Impl(item) => {
            for assoc_item in item.items() {
                traverse_item(cx, visitor, assoc_item.as_item())?;
            }
        },
        ItemKind::ExternBlock(item) => {
            for ext_item in item.items() {
                traverse_item(cx, visitor, ext_item.as_item())?;
            }
        },
        ItemKind::ExternCrate(_) | ItemKind::Use(_) | ItemKind::Unstable(_) | ItemKind::TyAlias(_) => {
            // These items have no sub nodes, which are visited by this visitor
        },
        _ => unreachable!("all items are covered"),
    }
    ControlFlow::Continue(())
}

pub fn traverse_body<'ast, B>(
    cx: &'ast MarkerContext<'ast>,
    visitor: &mut dyn Visitor<B>,
    body: &'ast Body<'ast>,
) -> ControlFlow<B> {
    visitor.visit_body(cx, body)?;

    traverse_expr(cx, visitor, body.expr())?;

    ControlFlow::Continue(())
}

pub fn traverse_stmt<'ast, B>(
    cx: &'ast MarkerContext<'ast>,
    visitor: &mut dyn Visitor<B>,
    stmt: StmtKind<'ast>,
) -> ControlFlow<B> {
    visitor.visit_stmt(cx, stmt)?;

    match stmt {
        StmtKind::Item(item) => {
            traverse_item(cx, visitor, item.item())?;
        },
        StmtKind::Let(lt) => {
            if let Some(init) = lt.init() {
                traverse_expr(cx, visitor, init)?;
            }
            if let Some(els) = lt.els() {
                traverse_expr(cx, visitor, els)?;
            }
        },
        StmtKind::Expr(expr) => {
            traverse_expr(cx, visitor, expr.expr())?;
        },
        _ => unreachable!("all statements are covered"),
    }

    ControlFlow::Continue(())
}

#[allow(clippy::too_many_lines)]
pub fn traverse_expr<'ast, B>(
    cx: &'ast MarkerContext<'ast>,
    visitor: &mut dyn Visitor<B>,
    expr: ExprKind<'ast>,
) -> ControlFlow<B> {
    visitor.visit_expr(cx, expr)?;

    match expr {
        ExprKind::Block(e) => {
            for stmt in e.stmts() {
                traverse_stmt(cx, visitor, *stmt)?;
            }
            if let Some(block_expr) = e.expr() {
                traverse_expr(cx, visitor, block_expr)?;
            }
        },
        ExprKind::Closure(e) => {
            if let VisitorScope::AllBodies = visitor.scope() {
                traverse_body(cx, visitor, cx.body(e.body_id()))?;
            }
        },
        ExprKind::UnaryOp(e) => {
            traverse_expr(cx, visitor, e.expr())?;
        },
        ExprKind::Ref(e) => {
            traverse_expr(cx, visitor, e.expr())?;
        },
        ExprKind::BinaryOp(e) => {
            traverse_expr(cx, visitor, e.left())?;
            traverse_expr(cx, visitor, e.right())?;
        },
        ExprKind::Try(e) => {
            traverse_expr(cx, visitor, e.expr())?;
        },
        ExprKind::Assign(e) => {
            traverse_expr(cx, visitor, e.value())?;
        },
        ExprKind::As(e) => {
            traverse_expr(cx, visitor, e.expr())?;
        },
        ExprKind::Call(e) => {
            traverse_expr(cx, visitor, e.operand())?;
            for arg in e.args() {
                traverse_expr(cx, visitor, *arg)?;
            }
        },
        ExprKind::Method(e) => {
            traverse_expr(cx, visitor, e.receiver())?;
            for arg in e.args() {
                traverse_expr(cx, visitor, *arg)?;
            }
        },
        ExprKind::Array(e) => {
            for el in e.elements() {
                traverse_expr(cx, visitor, *el)?;
            }
            if let Some(len) = e.len() {
                traverse_expr(cx, visitor, len.expr())?;
            }
        },
        ExprKind::Tuple(e) => {
            for el in e.elements() {
                traverse_expr(cx, visitor, *el)?;
            }
        },
        ExprKind::Ctor(e) => {
            for field in e.fields() {
                traverse_expr(cx, visitor, field.expr())?;
            }
            if let Some(base) = e.base() {
                traverse_expr(cx, visitor, base)?;
            }
        },
        // I like the simplicity of the API, even if the dereference part of
        // slices is a bit annoying. But typing all of this out is kind of meh.
        // not super interesting and almost just copy pasta, but not enough for
        // a macro... Oh well, back to work
        ExprKind::Range(e) => {
            if let Some(start) = e.start() {
                traverse_expr(cx, visitor, start)?;
            }
            if let Some(end) = e.end() {
                traverse_expr(cx, visitor, end)?;
            }
        },
        ExprKind::Index(e) => {
            traverse_expr(cx, visitor, e.operand())?;
            traverse_expr(cx, visitor, e.index())?;
        },
        ExprKind::Field(e) => {
            traverse_expr(cx, visitor, e.operand())?;
        },
        ExprKind::If(e) => {
            traverse_expr(cx, visitor, e.condition())?;
            traverse_expr(cx, visitor, e.then())?;
            if let Some(els) = e.els() {
                traverse_expr(cx, visitor, els)?;
            }
        },
        ExprKind::Let(e) => {
            traverse_expr(cx, visitor, e.scrutinee())?;
        },
        ExprKind::Match(e) => {
            traverse_expr(cx, visitor, e.scrutinee())?;
            for arm in e.arms() {
                if let Some(guard) = arm.guard() {
                    traverse_expr(cx, visitor, guard)?;
                }
                traverse_expr(cx, visitor, arm.expr())?;
            }
        },
        ExprKind::Break(e) => {
            if let Some(val) = e.expr() {
                traverse_expr(cx, visitor, val)?;
            }
        },
        ExprKind::Return(e) => {
            if let Some(val) = e.expr() {
                traverse_expr(cx, visitor, val)?;
            }
        },
        ExprKind::For(e) => {
            traverse_expr(cx, visitor, e.iterable())?;
            traverse_expr(cx, visitor, e.block())?;
        },
        ExprKind::Loop(e) => {
            traverse_expr(cx, visitor, e.block())?;
        },
        ExprKind::While(e) => {
            traverse_expr(cx, visitor, e.condition())?;
            traverse_expr(cx, visitor, e.block())?;
        },
        ExprKind::Await(e) => {
            traverse_expr(cx, visitor, e.expr())?;
        },
        ExprKind::IntLit(_)
        | ExprKind::FloatLit(_)
        | ExprKind::StrLit(_)
        | ExprKind::CharLit(_)
        | ExprKind::BoolLit(_)
        | ExprKind::Unstable(_)
        | ExprKind::Path(_)
        | ExprKind::Continue(_) => {
            // These expressions have no sub nodes, which are visited by this visitor
        },
        _ => unreachable!("all expressions are covered"),
    }

    ControlFlow::Continue(())
}

/// This trait is implemented for nodes, that can be traversed by a [`Visitor`].
pub trait Traversable<'ast, B>
where
    Self: Sized + Copy,
{
    /// This calls the `traverse_*` function for the implementing node.
    fn traverse(self, cx: &'ast MarkerContext<'ast>, visitor: &mut dyn Visitor<B>) -> ControlFlow<B>;

    /// This function calls the given closure for every expression in the node. This
    /// traversal will not enter any nested bodies.
    ///
    /// This function is a simple wrapper around the [`Visitor`] trait, that is good
    /// for most use cases. For example, the following code counts the number of `if`s
    /// in a body:
    ///
    /// ```
    /// # use marker_api::prelude::*;
    /// # use std::ops::ControlFlow;
    /// # use marker_utils::visitor::Traversable;
    /// fn count_ifs<'ast>(cx: &'ast MarkerContext<'ast>, body: &'ast Body<'ast>) -> u32 {
    ///     let mut count = 0;
    ///     let _: Option<()> = body.for_each_expr(
    ///         cx,
    ///         |expr| {
    ///             if matches!(expr, ExprKind::If(_)) {
    ///                 count += 1;
    ///             }
    ///             ControlFlow::Continue(())
    ///         }
    ///     );
    ///     count
    /// }
    /// ```
    fn for_each_expr<F: for<'a> FnMut(ExprKind<'a>) -> ControlFlow<B>>(
        self,
        cx: &'ast MarkerContext<'ast>,
        f: F,
    ) -> Option<B> {
        struct ExprVisitor<F> {
            f: F,
        }
        impl<B, F: for<'a> FnMut(ExprKind<'a>) -> ControlFlow<B>> Visitor<B> for ExprVisitor<F> {
            fn visit_expr<'v_ast>(
                &mut self,
                _cx: &'v_ast MarkerContext<'v_ast>,
                expr: ExprKind<'v_ast>,
            ) -> ControlFlow<B> {
                (self.f)(expr)
            }
        }
        let mut visitor = ExprVisitor { f };

        match self.traverse(cx, &mut visitor) {
            ControlFlow::Continue(()) => None,
            ControlFlow::Break(b) => Some(b),
        }
    }
}

/// This macro implements the [`Traversable`] trait for a given node that implements `Copy`
macro_rules! impl_traversable_for {
    ($ty:ty, $func:ident) => {
        impl<'ast, B> Traversable<'ast, B> for $ty {
            fn traverse(self, cx: &'ast MarkerContext<'ast>, visitor: &mut dyn Visitor<B>) -> ControlFlow<B> {
                $func(cx, visitor, self)
            }
        }
    };
}

impl_traversable_for!(ExprKind<'ast>, traverse_expr);
impl_traversable_for!(StmtKind<'ast>, traverse_stmt);
impl_traversable_for!(ItemKind<'ast>, traverse_item);
impl_traversable_for!(&'ast Body<'ast>, traverse_body);

/// This trait extends the [`Traversable`] trait with more functions, specific to
/// the `bool` return type.
pub trait BoolTraversable<'ast>: Traversable<'ast, bool> {
    /// Checks if the given node contains an early return, in the form of an
    /// [`ReturnExpr`](marker_api::ast::expr::ReturnExpr) or
    /// [`TryExpr`](marker_api::ast::expr::TryExpr).
    ///
    /// This function is useful, for lints which suggest moving code snippets into
    /// a closure or different function. Return statements might prevent the suggested
    /// refactoring.
    fn contains_return(&self, cx: &'ast MarkerContext<'ast>) -> bool {
        self.for_each_expr(cx, |expr| {
            if matches!(expr, ExprKind::Return(_) | ExprKind::Try(_)) {
                ControlFlow::Break(true)
            } else {
                ControlFlow::Continue(())
            }
        })
        .is_some()
    }
}

impl<'ast, T: Traversable<'ast, bool>> BoolTraversable<'ast> for T {}
