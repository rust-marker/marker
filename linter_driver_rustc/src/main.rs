#![warn(clippy::pedantic, clippy::index_refutable_slice)]

use linter_adapter::loader::ExternalLintPassRegistry;

fn main() {
    println!("Hello, world!");

    let mut registry = ExternalLintPassRegistry::default();
    registry.load_external_lib("./target/debug/liblinter_test.so").unwrap();
    registry.lint_passes.iter().for_each(|pass| pass.test_call("It works"));
}
