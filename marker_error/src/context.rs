use crate::Error;

/// Extension trait for `Result` and `Option` to add context to errors.
/// It is similar to `anyhow`'s `Context` trait, but it doesn't have a
/// separation between `.context()` and `.with_context()` methods, and
/// it has only a single one that always takes a closure.
pub trait Context<Ok, Kind> {
    /// Add a message to the error if it is [`Err`] or [`None`].
    /// This will produce a [`Result`] with the uncategorized error.
    ///
    /// When invoked on a [`Result`] type, the error of the result will be
    /// the source of the new error.
    fn context<Func, Str>(self, context: Func) -> Result<Ok, Error<Kind>>
    where
        Self: Sized,
        Func: FnOnce() -> Str,
        Str: Into<String>;
}

impl<Ok, Err, Kind> Context<Ok, Kind> for Result<Ok, Err>
where
    Err: std::error::Error + Send + Sync + 'static,
{
    fn context<Func, Str>(self, context: Func) -> Result<Ok, Error<Kind>>
    where
        Self: Sized,
        Func: FnOnce() -> Str,
        Str: Into<String>,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(source) => Err(Error::wrap(source, context())),
        }
    }
}

impl<Ok, Kind> Context<Ok, Kind> for Option<Ok> {
    fn context<Func, Str>(self, context: Func) -> Result<Ok, Error<Kind>>
    where
        Self: Sized,
        Func: FnOnce() -> Str,
        Str: Into<String>,
    {
        match self {
            Some(ok) => Ok(ok),
            None => Err(Error::root(context())),
        }
    }
}
