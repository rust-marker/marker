use crate::{
    ast::{ExprId, FieldId, ItemId, Span, StmtId, VariantId},
    context::AstContext,
    ffi::{FfiSlice, FfiStr},
    lint::Lint,
};

/// This builder creates the diagnostic object which will be emitted by the driver.
/// The documentation will showcase the messages in rustc's console emission style,
/// the actual display depends on the driver.
pub struct DiagnosticBuilder<'ast> {
    lint: &'static Lint,
    msg: String,
    node: EmissionNode,
    span: Span<'ast>,
    parts: Vec<DiagnosticPart<String, Span<'ast>>>,
}

impl<'ast> DiagnosticBuilder<'ast> {
    pub(crate) fn new(lint: &'static Lint, node: EmissionNode, msg: String, span: Span<'ast>) -> Self {
        Self {
            lint,
            msg,
            node,
            span,
            parts: vec![],
        }
    }

    /// This function adds a note to the diagnostic message. Notes are intended
    /// to provide additional context or explanations about the diagnostic.
    ///
    /// From rustc a text note would be displayed like this:
    /// ```text
    ///  warning: <lint message>
    ///  --> path/file.rs:1:1
    ///   |
    /// 1 | expression
    ///   | ^^^^^^^^^^
    ///   |
    ///   = note: <text>               <-- The note added by this function
    /// ```
    ///
    /// [`Self::span_note`] can be used to highlight a relevant [`Span`].
    pub fn note(&mut self, msg: impl ToString) {
        self.parts.push(DiagnosticPart::Note { msg: msg.to_string() });
    }

    /// This function adds a note with a [`Span`] to the diagnostic message.
    /// Spanned notes are intended to highlight relevant code snippets or
    /// help with explanations.
    ///
    /// From rustc a spanned note would be displayed like this:
    /// ```text
    ///  warning: <lint message>
    ///  --> path/file.rs:1:1
    ///   |
    /// 1 | expression
    ///   | ^^^^^^^^^^
    ///   |
    /// note: <text>                   <--
    ///  --> path/file.rs:2:1          <--
    ///   |                            <-- The spanned note added by this function
    /// 1 | context                    <--
    ///   | ^^^^^^^                    <--
    /// ```
    ///
    /// [`Self::note`] can be used to add text notes without a span.
    pub fn span_note(&mut self, msg: impl ToString, span: &Span<'ast>) {
        self.parts.push(DiagnosticPart::NoteSpan {
            msg: msg.to_string(),
            span: span.clone(),
        });
    }

    /// This function adds a help message. Help messages are intended to provide
    /// additional information about how the issue can be solved.
    ///
    /// From rustc a text help message would be displayed like this:
    /// ```text
    ///  warning: <lint message>
    ///  --> path/file.rs:1:1
    ///   |
    /// 1 | expression
    ///   | ^^^^^^^^^^
    ///   |
    ///   = help: <text>               <-- The help message added by this function
    /// ```
    ///
    /// [`Self::span_help`] can be used to highlight a relevant [`Span`].
    /// [`Self::span_suggestion`] can be used to add a help message with a suggestion.
    pub fn help(&mut self, msg: impl ToString) -> &mut Self {
        self.parts.push(DiagnosticPart::Help { msg: msg.to_string() });
        self
    }

    /// This function adds a help message with a [`Span`]. Spanned help messages
    /// are intended to highlight relevant code snippets that can be adapted to
    /// potentualy solve the problem.
    ///
    /// From rustc a spanned help message would be displayed like this:
    /// ```text
    ///  warning: <lint message>
    ///  --> path/file.rs:1:1
    ///   |
    /// 1 | expression
    ///   | ^^^^^^^^^^
    ///   |
    /// help: <text>                   <--
    ///  --> path/file.rs:2:1          <--
    ///   |                            <-- The spanned note added by this function
    /// 1 | code_to_change             <--
    ///   | ^^^^^^^^^^^^^^             <--
    /// ```
    ///
    /// [`Self::help`] can be used to add a text help message without a [`Span`].
    /// [`Self::span_suggestion`] can be used to add a help message with a suggestion.
    pub fn span_help(&mut self, msg: impl ToString, span: &Span<'ast>) {
        self.parts.push(DiagnosticPart::HelpSpan {
            msg: msg.to_string(),
            span: span.clone(),
        });
    }

    /// This function adds a spanned help message with a suggestion. The suggestion
    /// is a string which can be used to replace the marked [`Span`]. The confidence
    /// of a suggestion is expressed with the [`Applicability`].
    ///
    /// From rustc a suggestion would be displayed like this:
    /// ```text
    ///  warning: <lint message>
    ///  --> path/file.rs:1:1
    ///   |
    /// 1 | expression
    ///   | ^^^^^^^^^^ help: <msg>: `<suggestion>`      <-- The suggestion added by this function
    ///   |
    /// ```
    ///
    /// It's common to use `try` as a short suggestion message, if no further
    /// explanation is required.
    pub fn span_suggestion(
        &mut self,
        msg: impl ToString,
        span: &Span<'ast>,
        suggestion: impl ToString,
        app: Applicability,
    ) {
        self.parts.push(DiagnosticPart::Suggestion {
            msg: msg.to_string(),
            span: span.clone(),
            sugg: suggestion.to_string(),
            app,
        });
    }

    pub(crate) fn emit<'builder>(&'builder self, cx: &AstContext<'ast>) {
        let parts: Vec<_> = self
            .parts
            .iter()
            .map(|builder_part| builder_part.to_ffi_part())
            .collect();
        let diag = Diagnostic {
            lint: self.lint,
            msg: self.msg.as_str().into(),
            node: self.node,
            span: &self.span,
            parts: parts.as_slice().into(),
        };
        cx.emit_diagnostic(&diag)
    }
}

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum EmissionNode {
    Expr(ExprId),
    Item(ItemId),
    Stmt(StmtId),
    Field(FieldId),
    Variant(VariantId),
}

macro_rules! impl_into_emission_node_for {
    ($variant:ident, $ty:ty) => {
        impl From<$ty> for EmissionNode {
            fn from(value: $ty) -> Self {
                EmissionNode::$variant(value)
            }
        }

        impl From<&$ty> for EmissionNode {
            fn from(value: &$ty) -> Self {
                EmissionNode::$variant(*value)
            }
        }
    };
}

use impl_into_emission_node_for;

impl_into_emission_node_for!(Expr, ExprId);
impl_into_emission_node_for!(Item, ItemId);
impl_into_emission_node_for!(Stmt, StmtId);
impl_into_emission_node_for!(Field, FieldId);
impl_into_emission_node_for!(Variant, VariantId);

#[repr(C)]
#[non_exhaustive]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
pub(crate) enum DiagnosticPart<St, Sp> {
    Help {
        msg: St,
    },
    HelpSpan {
        msg: St,
        span: Sp,
    },
    Note {
        msg: St,
    },
    NoteSpan {
        msg: St,
        span: Sp,
    },
    Suggestion {
        msg: St,
        span: Sp,
        sugg: St,
        app: Applicability,
    },
}

impl<'ast> DiagnosticPart<String, Span<'ast>> {
    fn to_ffi_part<'part>(&'part self) -> DiagnosticPart<FfiStr<'part>, &'part Span<'ast>> {
        match self {
            DiagnosticPart::Help { msg } => DiagnosticPart::Help { msg: msg.into() },
            DiagnosticPart::HelpSpan { msg, span } => DiagnosticPart::HelpSpan { msg: msg.into(), span },
            DiagnosticPart::Note { msg } => DiagnosticPart::Note { msg: msg.into() },
            DiagnosticPart::NoteSpan { msg, span } => DiagnosticPart::NoteSpan { msg: msg.into(), span },
            DiagnosticPart::Suggestion { msg, span, sugg, app } => DiagnosticPart::Suggestion {
                msg: msg.into(),
                span,
                sugg: sugg.into(),
                app: *app,
            },
        }
    }
}

/// Indicates the confidence in the correctness of a suggestion.
///
/// All suggestions are marked with an `Applicability`. Tools use the applicability of a
/// suggestion to determine whether it should be automatically applied or if the user
/// should be consulted before applying the suggestion.
#[repr(C)]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
// FIXME: This will need to be updated according to rust-lang/rustfix#200
pub enum Applicability {
    /// The suggestion is definitely what the user intended, or maintains the exact
    /// meaning of the code. This suggestion should be automatically applied.
    ///
    /// In case of multiple `MachineApplicable` suggestions (whether as part of
    /// the same `multipart_suggestion` or not), all of them should be
    /// automatically applied.
    MachineApplicable,

    /// The suggestion may be what the user intended, but it is uncertain. The suggestion
    /// should result in valid Rust code if it is applied.
    MaybeIncorrect,

    /// The suggestion contains placeholders like `(...)` or `{ /* fields */ }`. The
    /// suggestion cannot be applied automatically because it will not result in
    /// valid Rust code. The user will need to fill in the placeholders.
    HasPlaceholders,

    /// The applicability of the suggestion is unknown.
    Unspecified,
}

/// This is the diagnostic object for the lint emission. It is constructed
/// with by the [`DiagnosticBuilder`].
#[repr(C)]
#[derive(Debug)]
#[cfg_attr(feature = "driver-api", visibility::make(pub))]
pub(crate) struct Diagnostic<'builder, 'ast> {
    pub lint: &'static Lint,
    pub msg: FfiStr<'builder>,
    pub node: EmissionNode,
    pub span: &'builder Span<'ast>,
    pub parts: FfiSlice<'builder, DiagnosticPart<FfiStr<'builder>, &'builder Span<'ast>>>,
}

impl<'builder, 'ast> Diagnostic<'builder, 'ast> {
    pub fn msg(&self) -> &str {
        self.msg.get()
    }
}
