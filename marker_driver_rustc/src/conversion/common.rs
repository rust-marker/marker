mod id;
pub use id::*;
mod span;
pub use span::*;
mod unstable;
pub use unstable::*;
mod ast_path;
pub use ast_path::*;

use marker_api::{ast::Abi, lint::Level};

#[must_use]
pub fn to_rustc_lint_level(api_level: Level) -> rustc_lint::Level {
    match api_level {
        Level::Allow => rustc_lint::Level::Allow,
        Level::Warn => rustc_lint::Level::Warn,
        Level::Deny => rustc_lint::Level::Deny,
        Level::Forbid => rustc_lint::Level::Forbid,
        _ => unreachable!(),
    }
}

#[must_use]
pub fn to_api_abi(rust_abi: rustc_target::spec::abi::Abi) -> Abi {
    match rust_abi {
        rustc_target::spec::abi::Abi::Rust => Abi::Default,
        rustc_target::spec::abi::Abi::C { .. } => Abi::C,
        _ => Abi::Other,
    }
}
