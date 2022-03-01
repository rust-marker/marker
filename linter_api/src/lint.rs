
#[derive(Debug)]
// This can sadly not be marked as #[non_exhaustive] as the struct construction
// has to be possible in a static context.
pub struct Lint {
    /// See <https://rustc-dev-guide.rust-lang.org/diagnostics.html#lint-naming>
    /// for naming guidelines.
    pub name: &'static str,
    /// Default level for the lint.
    ///
    /// See <https://rustc-dev-guide.rust-lang.org/diagnostics.html#diagnostic-levels>
    /// for guidelines on choosing a default level.
    pub default_level: Level,
    /// Warning, dealing with macros can be difficult. It's recommended to set this to `false`
    /// 
    /// FIXME: Here I would prefer an enum to enable users to also select between no
    /// linting, some linting in local macros and all linting in all macros.
    pub report_in_macro: bool,
    /// This text will be provided when the user if they call the explain function for
    /// the linter. Idealy it should have a maximum with of 80 characters. It can span
    /// over multiple lines.
    pub explaination: &'static str,
    // FIXME: Here I would also like a solution to add the driver specific lint instance to this struct.
    // Currently we have to maintain a map to do this conversion.
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Level {
    /// The lint is allowed. A created diagnostic will not be emitted to the user.
    /// This level can be overridden. It's usefull for rather strict lints.
    Allow,
    /// The `warn` level will produce a warning if the lint was violated, however the
    /// compiler will continue with its execution.
    ///
    /// This level might also be used in instances were the diagnostic is not emitted
    /// to the user but used internally. This can for instance happen for lint
    /// expectations (RFC 2383).
    Warn,
    /// The `deny` level will produce an error and stop further execution after the lint
    /// pass is complete.
    Deny,
}

#[macro_export]
macro_rules! declare_lint {
    ($(#[$attr:meta])* $NAME: ident, $LEVEL: ident, $EXPLAINATION: literal $(,)?) => {
        $crate::lint::declare_lint!{$(#[$attr])* $NAME, $LEVEL, $EXPLAINATION, false }
    };
    ($(#[$attr:meta])* $NAME: ident, $LEVEL: ident, $EXPLAINATION: literal, $REPORT_IN_MACRO: literal $(,)? ) => {
        $(#[$attr])*
        pub static $NAME: &$crate::lint::Lint = &$crate::lint::Lint {
            name: stringify!($NAME),
            default_level: $crate::lint::Level::$LEVEL,
            report_in_macro: $REPORT_IN_MACRO,
            explaination: $EXPLAINATION,
        };
    };
}

pub use declare_lint;
