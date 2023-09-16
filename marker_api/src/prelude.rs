//! This prelude is a collection of traits and types which are commonly used
//! when working with Marker. Simply add `use marker_api::prelude::*;` to your
//! file, to import them all.

// Traits:
pub use crate::ast::expr::ExprData;
pub use crate::ast::item::ItemData;
pub use crate::ast::pat::PatData;
pub use crate::ast::stmt::StmtData;
pub use crate::ast::ty::SynTyData;
pub use crate::diagnostic::EmissionNode;

// Common types
pub use crate::ast::expr::ExprKind;
pub use crate::ast::item::Body;
pub use crate::ast::item::ItemKind;
pub use crate::ast::pat::PatKind;
pub use crate::ast::stmt::StmtKind;
pub use crate::ast::ty::SemTyKind;
pub use crate::ast::ty::SynTyKind;
pub use crate::ast::Ident;
pub use crate::ast::Span;
pub use crate::AstContext;
