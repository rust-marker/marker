#![cfg_attr(
    marker,
    feature(register_tool),
    register_tool(marker),
    warn(marker::not_using_has_span_trait)
)]

use marker_api::prelude::*;

pub fn blackjack(_: u32, _: &Span<'_>) {}
pub fn rampage(_: &marker_api::span::Span<'_>, _: bool) {}

fn main() {}
