#[derive(Debug)]
// This can sadly not be marked as #[non_exhaustive] as the struct construction
// has to be possible in a static context.
#[doc(hidden)]
pub struct Lint {
    /// A string identifier for the lint.
    ///
    /// This identifies the lint in attributes and in command-line arguments.
    /// In those contexts it is always lowercase. This allows
    /// `declare_lint!()` invocations to follow the convention of upper-case
    /// statics without repeating the name.
    ///
    /// The name is written with underscores, e.g., "unused_imports".
    /// On the command line, underscores become dashes.
    ///
    /// See <https://rustc-dev-guide.rust-lang.org/diagnostics.html#lint-naming>
    /// for naming guidelines.
    pub name: &'static str,

    /// Default level for the lint.
    ///
    /// See <https://rustc-dev-guide.rust-lang.org/diagnostics.html#diagnostic-levels>
    /// for guidelines on choosing a default level.
    pub default_level: Level,

    /// Description of the lint or the issue it detects.
    ///
    /// e.g., "imports that are never used"
    pub desc: &'static str,

    /// Starting at the given edition, default to the given lint level. If this is
    /// `None`, then use `default_level`.
    pub edition_lint_opts: Option<(Edition, Level)>,

    /// The level of macro reporting.
    ///
    /// See `MacroReport` for the possible levels.
    pub report_in_macro: MacroReport,

    pub future_incompatible: Option<FutureIncompatibleInfo>,

    /// `Some` if this lint is feature gated, otherwise `None`.
    pub feature_gate: Option<&'static str>,

    pub crate_level_only: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
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
#[derive(Copy, Clone, Debug, PartialEq, Hash)]
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
    Allow,
    Warn,
    Deny,
    Forbid,
}

/// Extra information for a future incompatibility lint.
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub struct FutureIncompatibleInfo {
    /// e.g., a URL for an issue/PR/RFC or error code
    pub reference: &'static str,
    /// The reason for the lint used by diagnostics to provide
    /// the right help message
    pub reason: FutureIncompatibilityReason,
    /// Whether to explain the reason to the user.
    ///
    /// Set to false for lints that already include a more detailed
    /// explanation.
    pub explain_reason: bool,
}

/// The reason for future incompatibility
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum FutureIncompatibilityReason {
    /// This will be an error in a future release
    /// for all editions
    FutureReleaseError,
    /// This will be an error in a future release, and
    /// Cargo should create a report even for dependencies
    FutureReleaseErrorReportNow,
    /// Previously accepted code that will become an
    /// error in the provided edition
    EditionError(Edition),
    /// Code that changes meaning in some way in
    /// the provided edition
    EditionSemanticsChange(Edition),
}

/// The edition of the compiler. (See [RFC 2052](https://github.com/rust-lang/rfcs/blob/master/text/2052-epochs.md).)
#[derive(Clone, Copy, Hash, PartialEq, PartialOrd, Debug, Eq)]
#[non_exhaustive]
pub enum Edition {
    // Editions *must* be kept in order, oldest to newest.
    /// The 2015 edition
    Edition2015,
    /// The 2018 edition
    Edition2018,
    /// The 2021 edition
    Edition2021,
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
            name: stringify!($NAME),
            default_level: $crate::lint::Level::$LEVEL,
            desc: $EXPLAINATION,
            edition_lint_opts: None,
            report_in_macro: $REPORT_IN_MACRO,
            future_incompatible: None,
            feature_gate: None,
            crate_level_only: false,
        };
    };
}

pub use declare_lint;
