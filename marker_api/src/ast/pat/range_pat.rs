use crate::{ast::expr::ExprKind, ffi::FfiOption};

use super::CommonPatData;

/// A range expression, like these:
///
/// ```
/// 1..9;
/// 3..;
/// ..5;
/// ..;
/// 0..=1;
/// ```
#[repr(C)]
#[derive(Debug)]
pub struct RangePat<'ast> {
    data: CommonPatData<'ast>,
    start: FfiOption<ExprKind<'ast>>,
    end: FfiOption<ExprKind<'ast>>,
    is_inclusive: bool,
}

impl<'ast> RangePat<'ast> {
    pub fn start(&self) -> Option<ExprKind<'ast>> {
        self.start.copy()
    }

    pub fn end(&self) -> Option<ExprKind<'ast>> {
        self.end.copy()
    }

    pub fn is_inclusive(&self) -> bool {
        self.is_inclusive
    }
}

super::impl_pat_data!(RangePat<'ast>, Range);

#[cfg(feature = "driver-api")]
impl<'ast> RangePat<'ast> {
    pub fn new(
        data: CommonPatData<'ast>,
        start: Option<ExprKind<'ast>>,
        end: Option<ExprKind<'ast>>,
        is_inclusive: bool,
    ) -> Self {
        Self {
            data,
            start: start.into(),
            end: end.into(),
            is_inclusive,
        }
    }
}
