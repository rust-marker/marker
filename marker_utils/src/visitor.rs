use std::ops::ControlFlow;

use marker_api::{
    ast::{
        expr::ExprKind,
        item::{Body, EnumVariant, Field, ItemKind},
        stmt::StmtKind,
    },
    context::AstContext,
};

pub trait Visitor<B> {
    fn visit_item<'ast>(&mut self, _cx: &'ast AstContext<'ast>, _item: ItemKind<'ast>) -> ControlFlow<B> {
        ControlFlow::Continue(())
    }

    fn visit_field<'ast>(&mut self, _cx: &'ast AstContext<'ast>, _field: &'ast Field<'ast>) -> ControlFlow<B> {
        ControlFlow::Continue(())
    }

    fn visit_variant<'ast>(
        &mut self,
        _cx: &'ast AstContext<'ast>,
        _variant: &'ast EnumVariant<'ast>,
    ) -> ControlFlow<B> {
        ControlFlow::Continue(())
    }

    fn visit_body<'ast>(&mut self, _cx: &'ast AstContext<'ast>, _body: &'ast Body<'ast>) -> ControlFlow<B> {
        ControlFlow::Continue(())
    }

    fn visit_stmt<'ast>(&mut self, _cx: &'ast AstContext<'ast>, _stmt: StmtKind<'ast>) -> ControlFlow<B> {
        ControlFlow::Continue(())
    }

    fn visit_expr<'ast>(&mut self, _cx: &'ast AstContext<'ast>, _expr: ExprKind<'ast>) -> ControlFlow<B> {
        ControlFlow::Continue(())
    }
}

pub fn traverse_item<'ast, B>(
    cx: &'ast AstContext<'ast>,
    visitor: &mut dyn Visitor<B>,
    kind: ItemKind<'ast>,
) -> ControlFlow<B> {
    visitor.visit_item(cx, kind)?;
    match kind {
        ItemKind::Mod(module) => {
            for mod_item in module.items() {
                traverse_item(cx, visitor, *mod_item)?;
            }
        },
        ItemKind::Static(item) => {
            if let Some(body_id) = item.body_id() {
                traverse_body(cx, visitor, cx.body(body_id))?;
            }
        },
        ItemKind::Const(item) => {
            if let Some(body_id) = item.body_id() {
                traverse_body(cx, visitor, cx.body(body_id))?;
            }
        },
        ItemKind::Fn(item) => {
            if let Some(body_id) = item.body_id() {
                traverse_body(cx, visitor, cx.body(body_id))?;
            }
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
    cx: &'ast AstContext<'ast>,
    visitor: &mut dyn Visitor<B>,
    body: &'ast Body<'ast>,
) -> ControlFlow<B> {
    visitor.visit_body(cx, body)?;

    traverse_expr(cx, visitor, body.expr())?;

    ControlFlow::Continue(())
}

pub fn traverse_stmt<'ast, B>(
    cx: &'ast AstContext<'ast>,
    visitor: &mut dyn Visitor<B>,
    stmt: StmtKind<'ast>,
) -> ControlFlow<B> {
    visitor.visit_stmt(cx, stmt)?;

    match stmt {
        StmtKind::Item(item) => {
            traverse_item(cx, visitor, item)?;
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
            traverse_expr(cx, visitor, expr)?;
        },
        _ => unreachable!("all statements are covered"),
    }

    ControlFlow::Continue(())
}

#[allow(clippy::too_many_lines)]
pub fn traverse_expr<'ast, B>(
    cx: &'ast AstContext<'ast>,
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
            traverse_body(cx, visitor, cx.body(e.body_id()))?;
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
        ExprKind::QuestionMark(e) => {
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
            if let Some(end) = e.start() {
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
