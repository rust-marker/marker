#![doc = include_str!("../README.md")]

mod context;
mod trace;

pub use context::*;
pub use trace::*;

mod imp {
    /// This type is used when no additional kind information is needed for an error.
    /// It doesn't need to be referred outside of this crate, so it is under a private
    /// module, even if nominally it is public.
    #[derive(Debug, thiserror::Error, miette::Diagnostic)]
    pub enum Never {}
}

use imp::Never;
use miette::Diagnostic;
use std::fmt::{self, Debug};
use std::sync::Arc;

pub type Result<Ok = (), Kind = Never> = std::result::Result<Ok, Error<Kind>>;

/// Env var that enables capturing traces in errors.
pub(crate) const MARKER_ERROR_TRACE: &str = "MARKER_ERROR_TRACE";

/// A polymorphic error type that can optionally contain a strongly-typed kind
/// for structural error handling (when an error needs to be matched on).
/// It's possible to construct it using just a message string and no kind.
///
/// See the various constructor method on this type and the [`Context`]
/// extension trait for more details.
pub struct Error<Kind = Never> {
    /// Let's keep the error type of a pointer size. With `Arc` we may also
    /// make it cloneable if such need ever arises.
    imp: Arc<ErrorImpl<Kind>>,
}

struct ErrorImpl<Kind> {
    category: ErrorCategory<Kind>,

    /// Only enabled if the `error_trace` target is enabled in `MARKER_LOG`.
    trace: Option<ErrorTrace>,
}

/// The enum of all errors. It has special variants with `Uncategorized` in their
/// name which are used for general unrecoverable errors that we don't need to
/// match on.
///
/// If some kind of error needs to be handled specially, it needs to be
/// moved from `Uncategorized` to a dedicated enum variant in [`ErrorCategory::Categorized`].
/// For example, if you want to add a help message or for some other elaborate cases.
#[derive(Debug, thiserror::Error)]
enum ErrorCategory<Kind> {
    #[error(transparent)]
    Categorized(Kind),

    /// Used for errors that should just be propagated to the caller
    /// with an additional `message` context.
    #[error("{message}")]
    Uncategorized {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Used for errors that should just be propagated to the caller
    /// without any additional context.
    #[error(transparent)]
    TransparentUncategorized(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// Used for errors that should just be propagated to the caller
    /// and there are many related errors that should be propagated.
    #[error("{message}")]
    ManyUncategorized { message: String, errors: Vec<Error<Kind>> },
}

impl<Kind> Error<Kind> {
    /// Output the error to `stderr`. Use this to ultimately display it to
    /// the user in a top-level main function, for example.
    pub fn print(&self)
    where
        Kind: miette::Diagnostic,
    {
        eprint!("{}", ErrorRender(self));
    }

    /// Get a reference to the error's typed kind. If this contains an uncategorized
    /// error then [`None`] is returned.
    pub fn kind(&self) -> Option<&Kind> {
        match &self.imp.category {
            ErrorCategory::Categorized(kind) => Some(kind),
            _ => None,
        }
    }

    /// Create a new categorized error from the given kind.
    pub fn from_kind(kind: Kind) -> Self {
        Self::from_category(ErrorCategory::Categorized(kind))
    }

    fn from_category(category: ErrorCategory<Kind>) -> Self {
        // We don't use any closures to not add more stack frames to the backtrace
        let trace = if std::env::var(MARKER_ERROR_TRACE).is_ok() {
            Some(ErrorTrace::capture())
        } else {
            None
        };

        let imp = ErrorImpl { category, trace };

        Self { imp: Arc::new(imp) }
    }

    /// Wrap an existing error making it a source of this error, and add
    /// more context with a message.
    pub fn wrap(source: impl Into<Box<dyn std::error::Error + Send + Sync>>, message: impl Into<String>) -> Self {
        Self::from_category(ErrorCategory::Uncategorized {
            message: message.into(),
            source: Some(source.into()),
        })
    }

    /// Make a root error that doesn't have a source error
    pub fn root(message: impl Into<String>) -> Self {
        Self::from_category(ErrorCategory::Uncategorized {
            message: message.into(),
            source: None,
        })
    }

    /// Accepts an iterator of errors and returns `Ok` if it is empty,
    /// otherwise returns a single [`Error`] if it contains exactly one error,
    /// and returns an error with many [`Error`]s inside if there are more than one.
    pub fn try_many(errors: impl IntoIterator<Item = Self>, message: impl Into<String>) -> Result<(), Kind> {
        let mut errors = errors.into_iter();
        let Some(first) = errors.next() else {
            return Ok(());
        };

        let Some(second) = errors.next() else {
            return Err(first);
        };

        Err(Self::many([first, second].into_iter().chain(errors), message))
    }

    /// Many related errors happened. They don't form the chain of causality, but
    /// they are related to each other.
    pub fn many(errors: impl IntoIterator<Item = Error<Kind>>, message: impl Into<String>) -> Self {
        Self::from_category(ErrorCategory::ManyUncategorized {
            message: message.into(),
            errors: Vec::from_iter(errors),
        })
    }

    /// Make an uncategorized error without any additional context that will
    /// just transparently wrap the given error.
    pub fn transparent(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> Self {
        Self::from_category(ErrorCategory::TransparentUncategorized(error.into()))
    }
}

impl<Kind> fmt::Debug for Error<Kind>
where
    Kind: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.imp.category, f)
    }
}

impl<Kind> fmt::Display for Error<Kind>
where
    Kind: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.imp.category, f)?;

        if let Some(trace) = &self.imp.trace {
            write!(f, "\n{trace}")?;
        }

        Ok(())
    }
}

impl<Kind> std::error::Error for Error<Kind>
where
    Kind: std::error::Error,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.imp.category.source()
    }
}

/// Forward the [`Diagnostic`] implementation to the categorized error.
impl<Kind> Diagnostic for Error<Kind>
where
    Kind: Diagnostic,
{
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.kind().and_then(Diagnostic::code)
    }

    fn severity(&self) -> Option<miette::Severity> {
        self.kind().and_then(Diagnostic::severity)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.kind().and_then(Diagnostic::help)
    }

    fn url<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        self.kind().and_then(Diagnostic::url)
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.kind().and_then(Diagnostic::source_code)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.kind().and_then(Diagnostic::labels)
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        match &self.imp.category {
            ErrorCategory::Categorized(kind) => kind.related(),
            ErrorCategory::ManyUncategorized { errors, .. } => Some(Box::new(errors.iter().map(|err| err as _))),
            ErrorCategory::Uncategorized { .. } | ErrorCategory::TransparentUncategorized(_) => None,
        }
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.kind().and_then(Diagnostic::diagnostic_source)
    }
}

/// Used to render the error using the fancy output implemented by [`miette`]
struct ErrorRender<'a, Kind>(&'a Error<Kind>);

impl<Kind> std::fmt::Display for ErrorRender<'_, Kind>
where
    Kind: miette::Diagnostic,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let handler = miette::MietteHandlerOpts::new().width(140).build();

        miette::ReportHandler::debug(&handler, self.0, f)
    }
}

impl<Kind> From<Kind> for Error<Kind> {
    fn from(value: Kind) -> Self {
        Self::from_kind(value)
    }
}
