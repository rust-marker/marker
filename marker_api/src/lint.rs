#[repr(C)]
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
    pub explanation: &'static str,

    /// The level of macro reporting.
    ///
    /// See [`MacroReport`] for the possible levels.
    pub report_in_macro: MacroReport,
    // FIXME: We might want to add more fields. This should be possible as this
    // struct is always constructed by a macro controlled by marker. These are some
    // additional fields used  in rustc:
    // * pub edition_lint_opts: Option<(Edition, Level)>,
    // * pub future_incompatible: Option<FutureIncompatibleInfo>,
    // * pub feature_gate: Option<&'static str>,
    // * pub crate_level_only: bool,
}

/// FIXME(xFrednet): These settings currently don't work.
///
/// See rust-marker#149
#[repr(C)]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum MacroReport {
    /// No reporting in local or external macros.
    No,
    /// Only report in local macros.
    Local,
    /// Report in local and external macros.
    All,
}

/// Setting for how to handle a lint.
#[repr(C)]
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
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
    ($(#[$attr:meta])* $NAME: ident, $LEVEL: ident, $EXPLANATION: literal $(,)?) => {
        $crate::declare_lint!{$(#[$attr])* $NAME, $LEVEL, $EXPLANATION, $crate::lint::MacroReport::No }
    };
    ($(#[$attr:meta])* $NAME: ident, $LEVEL: ident,
        $EXPLANATION: literal, $REPORT_IN_MACRO: expr $(,)?
    ) => {
        $(#[$attr])*
        pub static $NAME: &$crate::lint::Lint = &$crate::lint::Lint {
            name: concat!("marker::", stringify!($NAME)),
            default_level: $crate::lint::Level::$LEVEL,
            explanation: $EXPLANATION,
            report_in_macro: $REPORT_IN_MACRO,
        };
    };
}
