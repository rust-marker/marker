#[derive(Debug, PartialEq, Eq, Hash)]
// This sadly cannot be marked as #[non_exhaustive] as the struct construction
// has to be possible in a static context.
#[doc(hidden)]
pub struct Lint {
    /// A string identifier for the lint.
    ///
    /// This identifies the lint in attributes and in command-line arguments.
    /// In those contexts it is always lowercase. This allows
    /// [`declare_lint!`] macro invocations to follow the convention of upper-case
    /// statics without repeating the name.
    ///
    /// The name is written with underscores, e.g., "unused_imports".
    /// On the command line, underscores become dashes.
    ///
    /// See <https://rustc-dev-guide.rust-lang.org/diagnostics.html#lint-naming>
    /// for naming guidelines.
    ///
    /// [`declare_lint!`]: declare_lint
    pub name: &'static str,

    /// Default level for the lint.
    ///
    /// See <https://rustc-dev-guide.rust-lang.org/diagnostics.html#diagnostic-levels>
    /// for guidelines on choosing a default level.
    pub default_level: Level,

    /// Description of the lint or the issue it detects.
    ///
    /// e.g., "imports that are never used"
    pub explaination: &'static str,

    /// The level of macro reporting.
    ///
    /// See [`MacroReport`] for the possible levels.
    pub report_in_macro: MacroReport,
    // TODO: do we want these
    // pub edition_lint_opts: Option<(Edition, Level)>,
    // pub future_incompatible: Option<FutureIncompatibleInfo>,
    // pub feature_gate: Option<&'static str>,
    // pub crate_level_only: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MacroReport {
    /// No reporting in local or external macros.
    No,
    /// Only report in local macros.
    Local,
    /// Report in local and external macros.
    All,
}

/// Indicates the confidence in the correctness of a suggestion.
///
/// All suggestions are marked with an `Applicability`. Tools use the applicability of a
/// suggestion to determine whether it should be automatically applied or if the user
/// should be consulted before applying the suggestion.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
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

/// Setting for how to handle a lint.
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
#[non_exhaustive]
pub enum Level {
    /// The lint is allowed. A created diagnostic will not be emitted to the user by default.
    /// This level can be overridden. It's useful for rather strict lints.
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
    /// The `forbid` level will produce an error and cannot be overriden by the user.
    ///
    /// Choosing this diagnostic level should require heavy consideration, because should a lint
    /// with this level produce a false-positive, the user won't have an option to `allow` the lint
    /// for this particular case, and will be forced to either:
    /// - Write wrong code just to satisfy the lint
    /// - Remove the whole lint crate
    ///
    /// To produce an error, but make the lint possible to override see [`Deny`](`Self::Deny`).
    Forbid,
}

#[macro_export]
macro_rules! declare_lint {
    ($(#[$attr:meta])* $NAME: ident, $LEVEL: ident, $EXPLAINATION: literal $(,)?) => {
        $crate::lint::declare_lint!{$(#[$attr])* $NAME, $LEVEL, $EXPLAINATION, $crate::lint::MacroReport::No }
    };
    ($(#[$attr:meta])* $NAME: ident, $LEVEL: ident,
        $EXPLAINATION: literal, $REPORT_IN_MACRO: expr $(,)?
    ) => {
        $(#[$attr])*
        pub static $NAME: &$crate::lint::Lint = &$crate::lint::Lint {
            name: concat!("marker::", stringify!($NAME)),
            default_level: $crate::lint::Level::$LEVEL,
            explaination: $EXPLAINATION,
            report_in_macro: $REPORT_IN_MACRO,
        };
    };
}

pub use declare_lint;
