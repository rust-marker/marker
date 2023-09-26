//! This module is responsible for the construction of diagnostic messages. The
//! [`DiagnosticBuilder`] is the public stable interface, to construct messages.

use std::fmt::Debug;

use crate::{
    ast::{HasNodeId, NodeId},
    context::{with_cx, MarkerContext},
    ffi::{FfiSlice, FfiStr},
    lint::Lint,
    prelude::{HasSpan, Span},
};

/// This builder creates the diagnostic object which will be emitted by the driver.
/// The documentation will showcase the messages in rustc's console emission style,
/// the actual display depends on the driver.
pub struct DiagnosticBuilder<'ast> {
    /// This field will be `Some` if the created diagnostic will be emitted, otherwise
    /// it'll be `None`. See [`DiagnosticBuilder::decorate`] for more information, when the
    /// lint might be suppressed.
    inner: Option<DiagnosticBuilderInner<'ast>>,
}

struct DiagnosticBuilderInner<'ast> {
    lint: &'static Lint,
    node: NodeId,
    msg: String,
    span: Span<'ast>,
    parts: Vec<DiagnosticPart<String, Span<'ast>>>,
}

impl<'ast> DiagnosticBuilder<'ast> {
    /// Creates a new dummy builder, which basically makes all operations a noop
    pub(crate) fn dummy() -> Self {
        Self { inner: None }
    }

    pub(crate) fn new(lint: &'static Lint, node: NodeId, msg: String, span: Span<'ast>) -> Self {
        Self {
            inner: Some(DiagnosticBuilderInner {
                lint,
                msg,
                node,
                span,
                parts: vec![],
            }),
        }
    }

    /// This function sets the main [`Span`] of the created diagnostic.
    /// [`MarkerContext::emit_lint`] will by default use the [`Span`] of the given
    /// [`EmissionNode`].
    ///
    /// From rustc a lint emission would look like this:
    /// ```text
    ///  warning: <lint message>
    ///  --> path/file.rs:1:1
    ///   |
    /// 1 | node
    ///   | ^^^^                 <-- The main span set by this function
    ///   |
    /// ```
    pub fn span(&mut self, span: &Span<'ast>) -> &mut Self {
        if let Some(inner) = self.inner.as_mut() {
            inner.span = span.clone();
        }

        self
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
    pub fn note(&mut self, msg: impl Into<String>) -> &mut Self {
        if let Some(inner) = self.inner.as_mut() {
            inner.parts.push(DiagnosticPart::Note { msg: msg.into() });
        }

        self
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
    pub fn span_note(&mut self, msg: impl Into<String>, span: &Span<'ast>) -> &mut Self {
        if let Some(inner) = self.inner.as_mut() {
            inner.parts.push(DiagnosticPart::NoteSpan {
                msg: msg.into(),
                span: span.clone(),
            });
        }

        self
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
    pub fn help(&mut self, msg: impl Into<String>) -> &mut Self {
        if let Some(inner) = self.inner.as_mut() {
            inner.parts.push(DiagnosticPart::Help { msg: msg.into() });
        }

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
    pub fn span_help(&mut self, msg: impl Into<String>, span: &Span<'ast>) -> &mut Self {
        if let Some(inner) = self.inner.as_mut() {
            inner.parts.push(DiagnosticPart::HelpSpan {
                msg: msg.into(),
                span: span.clone(),
            });
        }

        self
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
        msg: impl Into<String>,
        span: &Span<'ast>,
        suggestion: impl Into<String>,
        app: Applicability,
    ) -> &mut Self {
        if let Some(inner) = self.inner.as_mut() {
            inner.parts.push(DiagnosticPart::Suggestion {
                msg: msg.into(),
                span: span.clone(),
                sugg: suggestion.into(),
                app,
            });
        }

        self
    }

    /// The `decorate` parameter accepts a closure, that is only executed, when the
    /// lint will actually be emitted in the end. Having them in a conditional closure
    /// will speedup the linting process if the lint is suppressed.
    ///
    /// A lint emission might be suppressed, if the lint is allowed at the
    /// [`EmissionNode`] or if the [`MacroReport`](crate::lint::MacroReport) level
    /// specified in the [`Lint`] isn't sufficient for context of the [`EmissionNode`].
    ///
    /// ```
    /// # use marker_api::prelude::*;
    /// # marker_api::declare_lint!{
    /// #     /// Dummy
    /// #     LINT,
    /// #     Warn,
    /// # };
    /// # fn value_provider<'ast>(cx: &MarkerContext<'ast>, node: ExprKind<'ast>) {
    ///     cx.emit_lint(LINT, node, "<lint message>").decorate(|diag| {
    ///         // This closure is only called, if the diagnostic will be emitted.
    ///         // Here you can create a beautiful help message.
    ///         diag.help("<text>");
    ///     });
    /// # }
    /// ```
    ///
    /// You can also checkout [`DiagnosticBuilder::done()`] to use a closure, without
    /// curly brackets.
    pub fn decorate<F>(&mut self, decorate: F) -> &mut Self
    where
        F: FnOnce(&mut DiagnosticBuilder<'ast>),
    {
        if self.inner.is_some() {
            decorate(self);
        }

        self
    }

    /// This function simply consumes the builder reference, which allows simpler
    /// use in closures. The following closures for [`DiagnosticBuilder::decorate`]
    /// are equivalent:
    ///
    /// ```
    /// # use marker_api::prelude::*;
    /// # marker_api::declare_lint!{
    /// #     /// Dummy
    /// #     LINT,
    /// #     Warn,
    /// # }
    /// # fn value_provider<'ast>(cx: &MarkerContext<'ast>, node: ExprKind<'ast>) {
    ///     // Without `done()`
    ///     cx.emit_lint(LINT, node, "<text>").decorate(|diag| {
    ///         diag.help("<text>");
    ///     });
    ///
    ///     // With `done()`
    ///     cx.emit_lint(LINT, node, "<text>").decorate(|diag| diag.help("<help>").done());
    /// # }
    /// ```
    pub fn done(&self) {}

    pub(crate) fn emit<'builder>(&'builder self, cx: &MarkerContext<'ast>) {
        if let Some(inner) = &self.inner {
            let parts: Vec<_> = inner.parts.iter().map(DiagnosticPart::to_ffi_part).collect();
            let diag = Diagnostic {
                lint: inner.lint,
                msg: inner.msg.as_str().into(),
                node: inner.node,
                span: &inner.span,
                parts: parts.as_slice().into(),
            };
            cx.emit_diagnostic(&diag);
        }
    }
}

impl<'ast> Drop for DiagnosticBuilder<'ast> {
    fn drop(&mut self) {
        with_cx(self, |cx| self.emit(cx));
    }
}

/// Every lint emission is bound to a specific node. The node is used to
/// determine the lint level and [`Span`] that is used for the main diagnostic
/// message.
pub trait EmissionNode<'ast>: Debug + HasSpan<'ast> + HasNodeId {}

impl<'ast, N: Debug + HasSpan<'ast> + HasNodeId> EmissionNode<'ast> for N {}

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
    pub node: NodeId,
    pub span: &'builder Span<'ast>,
    pub parts: FfiSlice<'builder, DiagnosticPart<FfiStr<'builder>, &'builder Span<'ast>>>,
}

impl<'builder, 'ast> Diagnostic<'builder, 'ast> {
    pub fn msg(&self) -> &str {
        self.msg.get()
    }
}
