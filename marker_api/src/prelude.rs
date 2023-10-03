//! This prelude is a collection of traits and types which are commonly used
//! when working with Marker. Simply add `use marker_api::prelude::*;` to your
//! file, to import them all.

// Traits:
pub use crate::ast::ExprData;
pub use crate::ast::ItemData;
pub use crate::ast::PatData;
pub use crate::ast::StmtData;
pub use crate::ast::TyData;
pub use crate::common::HasNodeId;
pub use crate::diagnostic::EmissionNode;
pub use crate::span::HasSpan;

// modules:
pub use crate::ast;
pub use crate::sem;

// IDs
pub use crate::common::{BodyId, ExprId, FieldId, GenericId, ItemId, NodeId, StmtId, TyDefId, VarId, VariantId};

// Common types
pub use crate::ast::ExprKind;
pub use crate::ast::ItemKind;
pub use crate::ast::PatKind;
pub use crate::ast::StmtKind;
pub use crate::lint::Lint;
pub use crate::span::Ident;
pub use crate::span::Span;
pub use crate::MarkerContext;
