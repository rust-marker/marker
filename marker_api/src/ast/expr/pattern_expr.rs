//! This module contains all expressions, which are typically used to construct
//! or deconstruct data. A simple example is the [`ArrayExpr`] which can be
//! used to create or destruct an array.

use crate::{
    ast::{AstQPath, Ident, Span, SpanId},
    context::with_cx,
    ffi::{FfiOption, FfiSlice},
};

use super::{CommonExprData, ExprKind, ExprPrecedence};

/// An array expressions can be used to construct an array or destruct an array.
///
/// ```
/// //            vvvvvvvvvvvv An array expression with four element expressions
/// let array_1 = [1, 2, 3, 4];
/// //            vvvvvv An array expression with one element and one len expression
/// let array_2 = [6; 3];
///
/// //  vvvvvvvvv An array expression destructing `array_2`
/// let [a, b, c] = array_2;
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct ArrayExpr<'ast> {
    data: CommonExprData<'ast>,
    elements: FfiSlice<'ast, ExprKind<'ast>>,
    len_expr: FfiOption<ExprKind<'ast>>,
}

impl<'ast> ArrayExpr<'ast> {
    pub fn elements(&self) -> &[ExprKind<'ast>] {
        self.elements.get()
    }

    pub fn len_expr(&self) -> Option<ExprKind<'ast>> {
        self.len_expr.copy()
    }
}

super::impl_expr_data!(
    ArrayExpr<'ast>,
    Array,
    fn precedence(&self) -> ExprPrecedence {
        ExprPrecedence::Pattern
    }
);

#[cfg(feature = "driver-api")]
impl<'ast> ArrayExpr<'ast> {
    pub fn new(
        data: CommonExprData<'ast>,
        elem_exprs: &'ast [ExprKind<'ast>],
        len_expr: Option<ExprKind<'ast>>,
    ) -> Self {
        Self {
            data,
            elements: elem_exprs.into(),
            len_expr: len_expr.into(),
        }
    }
}

/// A tuple expression used to construct or deconstruct a tuple.
///
/// ```
/// //          vvvvvvvvvvvv A tuple expression with four elements
/// let slice = (1, 2, 3, 4);
///
/// //  vvvvvvvvvvvv A tuple expression destructing `slice`
/// let (a, b, c, _) = slice;
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct TupleExpr<'ast> {
    data: CommonExprData<'ast>,
    elements: FfiSlice<'ast, ExprKind<'ast>>,
}

impl<'ast> TupleExpr<'ast> {
    pub fn elements(&self) -> &[ExprKind<'ast>] {
        self.elements.get()
    }
}

super::impl_expr_data!(
    TupleExpr<'ast>,
    Tuple,
    fn precedence(&self) -> ExprPrecedence {
        ExprPrecedence::Pattern
    }
);

#[cfg(feature = "driver-api")]
impl<'ast> TupleExpr<'ast> {
    pub fn new(data: CommonExprData<'ast>, elements: FfiSlice<'ast, ExprKind<'ast>>) -> Self {
        Self { data, elements }
    }
}

/// An expression constructing structs, unions and enum variants. For tuple
/// constructors, the field names will correspond to the field index.
///
/// ```
/// # #[derive(Debug, Default)]
/// # struct FieldStruct {
/// #     a: u32,
/// #     b: u32,
/// # }
/// # #[derive(Default)]
/// # struct TupleStruct(u32, u32);
/// # union Union {
/// #     a: u32,
/// # }
/// # enum Enum {
/// #     A,
/// #     B(u32),
/// #     C { f1: u32, f2: u32 },
/// # }
///
/// let _ = FieldStruct { a: 1, b: 2 };
/// //      ^^^^^^^^^^^^^^^^^^^^^^^^^^ A field struct constructor with two fields
/// let _ = FieldStruct { a: 10, ..FieldStruct::default() };
/// //      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
/// //      A field struct constructor with an optional base expression
///
/// let _ = Union { a: 8 };
/// //      ^^^^^^^^^^^^^ A union constructor with one field
///
/// let _ = TupleStruct { 0: 3, 1: 9 };
/// //      ^^^^^^^^^^^^^^^^^^^^^^^^^^ A tuple struct constructor with two fields
/// let _ = TupleStruct(1, 2);
/// //      ^^^^^^^^^^^^^^^^^ A tuple constructor with two elements, represented
/// //                        with field names, as above.
/// let _ = TupleStruct { 0: 3, ..TupleStruct::default() };
/// //      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
/// //      A tuple struct constructor with an optional base expression
///
/// let _ = Enum::A;
/// //      ^^^^^^^ An enum variant constructor without any elements
/// let _ = Enum::B(1);
/// //      ^^^^^^^^^^ An enum variant constructor with two elements
/// let _ = Enum::C { f1: 44, f2: 55 };
/// //      ^^^^^^^^^^^^^^^^^^^^^^^^^^ An enum variant constructor with named fields
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct CtorExpr<'ast> {
    data: CommonExprData<'ast>,
    path: AstQPath<'ast>,
    fields: FfiSlice<'ast, CtorField<'ast>>,
    base: FfiOption<ExprKind<'ast>>,
}

impl<'ast> CtorExpr<'ast> {
    /// The path will point to the
    pub fn path(&self) -> &AstQPath<'ast> {
        &self.path
    }

    pub fn fields(&self) -> &'ast [CtorField<'ast>] {
        self.fields.get()
    }

    pub fn base(&self) -> Option<ExprKind<'ast>> {
        self.base.copy()
    }
}

super::impl_expr_data!(
    CtorExpr<'ast>,
    Ctor,
    fn precedence(&self) -> ExprPrecedence {
        ExprPrecedence::Ctor
    }
);

#[cfg(feature = "driver-api")]
impl<'ast> CtorExpr<'ast> {
    pub fn new(
        data: CommonExprData<'ast>,
        path: AstQPath<'ast>,
        fields: &'ast [CtorField<'ast>],
        base: Option<ExprKind<'ast>>,
    ) -> Self {
        Self {
            data,
            path,
            fields: fields.into(),
            base: base.into(),
        }
    }
}

/// A single field inside a [`CtorExpr`].
#[repr(C)]
#[derive(Debug)]
pub struct CtorField<'ast> {
    span: SpanId,
    ident: Ident<'ast>,
    expr: ExprKind<'ast>,
}

impl<'ast> CtorField<'ast> {
    /// This returns the span of the entire field expression
    pub fn span(&self) -> &Span<'ast> {
        with_cx(self, |cx| cx.get_span(self.span))
    }

    /// The identifier of the field.
    pub fn ident(&self) -> &Ident<'ast> {
        &self.ident
    }

    pub fn expr(&self) -> ExprKind<'ast> {
        self.expr
    }
}

#[cfg(feature = "driver-api")]
impl<'ast> CtorField<'ast> {
    pub fn new(span: SpanId, ident: Ident<'ast>, expr: ExprKind<'ast>) -> Self {
        Self { span, ident, expr }
    }
}
