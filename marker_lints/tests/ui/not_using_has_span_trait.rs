#![cfg_attr(marker, warn(marker::marker_lints::not_using_has_span_trait))]

use marker_api::prelude::*;

pub fn blackjack(_: u32, _: &Span<'_>) {}
pub fn rampage(_: &marker_api::span::Span<'_>, _: bool) {}

fn main() {}
