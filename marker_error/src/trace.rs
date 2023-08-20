use std::backtrace::{Backtrace, BacktraceStatus};
use std::fmt;
use tracing_error::SpanTrace;
use yansi::Paint;

/// Contains a verbose information useful for debugging. The developers
/// will most likely want to see this when a panic happens in their code or
/// an error is returned from a function where it shouldn't be returned.
#[derive(Debug)]
pub struct ErrorTrace {
    spantrace: SpanTrace,
    backtrace: Backtrace,
}

impl ErrorTrace {
    pub fn capture() -> Self {
        Self {
            spantrace: SpanTrace::capture(),
            backtrace: Backtrace::force_capture(),
        }
    }

    fn write_trace(f: &mut fmt::Formatter<'_>, kind: &str, trace: &dyn fmt::Display) -> fmt::Result {
        writeln!(f, "{}\n{trace}", format_args!("{kind}:").red().bold())
    }
}

impl fmt::Display for ErrorTrace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let backtrace = &self.backtrace;

        match backtrace.status() {
            BacktraceStatus::Captured => Self::write_trace(f, "Backtrace", backtrace)?,
            // This must never happen because we use `force_capture`, but let's
            // be safe and not panic the second time in a row here if we this ever breaks
            BacktraceStatus::Disabled => writeln!(
                f,
                "{}",
                "⚠️  Backtrace wasn't captured. t is probably a bug in our error \
                trace capturing code."
                    .red(),
            )?,
            BacktraceStatus::Unsupported => writeln!(f, "{}", "backtrace is unsupported".red())?,
            _ => writeln!(f, "{}\n{backtrace}", "Backtrace is in unknown status:".red())?,
        };

        let spantrace = &self.spantrace;

        match spantrace.status() {
            tracing_error::SpanTraceStatus::CAPTURED => {
                write!(f, "{}\n{spantrace}", "Spantrace:".red().bold())?;
            },
            tracing_error::SpanTraceStatus::EMPTY => {
                write!(
                    f,
                    "{}. It may be expected if the error happened outside of \
                    any spans, or those spans were not enabled. You may try \
                    capturing it by enabling {} logging or increasing its level.",
                    "⚠️  Spantrace wasn't captured".yellow(),
                    "tracing".bold()
                )?;
            },
            tracing_error::SpanTraceStatus::UNSUPPORTED => f.write_str("spantrace is unsupported")?,
        }

        Ok(())
    }
}
