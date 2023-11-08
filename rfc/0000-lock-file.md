- Feature Name: `lock-file`
- Start Date: 2023-11-05
- RFC PR: [#307](https://github.com/rust-marker/marker/pull/307)
- Issue: [#251](https://github.com/rust-marker/marker/issues/251)

# Summary

Provide a `Marker.lock` file that is used to pin the versions of the dependencies of the compiled lint crates.

# Motivation

Today `Cargo.lock` file for the internal cargo workspace created for building the lint crates isn't exposed to the users of Marker. This means that every time when `cargo marker` runs on a fresh CI environment the latest compatible versions of lint crates and their dependencies are built. This poses several problems outlined below. This RFC intends to propose a solution for them.

## Compatibility fiasco

The real world is cruel. People may release a breaking change by mistake. Even `cargo-semver-checks` can't catch all possible semver compatibility breakages. Moreover, it is enough to say that not everyone uses it and people often make mistakes.

Some crate could also mistakenly or out-of-no-better-option rely on the behavior not included in its dependency semver guarantees. For example, some crates don't bump their major version if they increase the MSRV, or there are crates that simply don't care about the MSRV. There are so many reasons one could break the semver compatibility.

## Security

One may not want to always use the latest version of the dependencies. That increases the risk that any of the dependencies in the tree introduces a malicious build script in the new patch version that leaks CI credentials, for example.

## Licensing

There are a lot of different software licenses available in the wild. Their formulations can pose arbitrary requirements for the usage of the licensed code. Users should know if the dependencies of the lint crates that they compile pose any restrictions on their usage. If Marker always uses the latest compatible versions of the dependencies, then dependencies become highly dynamic. For example, a 3-rd party crate may add a new dependency with the license unacceptable for the end user. Without any lock file it is almost impossible to track which licenses appear in the dependency tree. Tools like `cargo deny` and `cargo audit` need a lock file to inspect the exact version of the crates in the dependency tree.

Licensing terms can also change in the new versions of the dependencies. Moreover, they can change so drastically that the users may want to cease using the dependency. If Marker always uses the latest compatible versions of the dependencies then it may start using the version of the dependency with unacceptable licensing terms.

## Caching the cargo home on CI

To save on the network bandwidth it's possible to run `cargo fetch` for the cargo workspace on one runner and share the fetched `$CARGO_HOME` with other runners via caching. If Marker's lint crates and their dependencies aren't in the `Cargo.lock` file, then `cargo fetch` won't fetch them making it not possible to run lint crate compilations with a `--frozen` flag on CI that prohibits networks access.

# Guide-level explanation

## What is a lint crate?

A lint crate is a cargo package that contains a single static library target and uses `marker_api` library to define the lints and the lint passes.

The code of the crate should look approximately like in the example below. This isn't its final form and it will change when it comes to implementation. What's unfortunate, is that this code uses proc macros. They provide a nicer and more consistent syntax than functional-like macros, they are much more powerful than declarative ones, but they increase the from-scratch compilation time. I think this is acceptable given that, for example, providing configuration capabilities would be too complicated to implement without proc macros (and `serde` maybe?).

The syntax shape and code organization here is designed to take into account the future possibility of [lint configurations](https://github.com/rust-marker/marker/issues/88), and exposing lints for [static linking](https://github.com/rust-marker/marker/issues/258).

**Example:**

```rust
// lib.rs

// This macro is required to enable the compilation of this crate as a `cdylib`
// when it is compiled in isolation.
marker_api::lint_crate! {}

// Create a module per each lint
mod lint_1;
mod lint_2;
```

```rust
// lint_1.rs
use marker_api::prelude::*;

// Generates an implementation of a `LintPassBase` trait that has a method
// to construct the lint pass from a string of a TOML section passed via FFI. If
// config parsing fails returns an error.
//
// Not sure what to do with the visibility of this struct. Maybe it would be useful
// to have it as `pub` for static linking scenario.
#[lint_pass]
struct LintPass<'ast> {
    // This makes it more convenient. Now methods on `LintPass` don't need to
    // have a separate `MarkerContext` parameter, but can just use `self` only.
    marker: MarkerContext<'ast>,

    /// Docs for the lint
    // The field name defines the lint name
    #[lint(level = "warn", macro_report = "off")]
    lint_a: Lint,

    /// Docs for the lint
    #[lint(level = "allow")]
    lint_b: Lint,

    /// Docs for the lint
    #[deprecated = "Describe why this lint was deprecated"]
    #[lint(level = "warn")]
    lint_c: Lint,

    // Optional
    // The `section` is optional when there is just a single lint in this lint pass.
    // In this case the `section` is equal to the lint name by default.
    //
    // The `section` is required when there are two or more lints because the config
    // is assumed to be shared among them the author of the lint pass needs to come
    // up with a name for the config section that will be common for all lints in
    // the lint pass.
    //
    // The docs generated for the lints defined in this lint pass will mention that they
    // can be configured with key-values defined in the configs declared in this struct.
    #[config(section = ".config_section_name")]
    local_config: Config,

    // This is a syntax to have a common configuration that is shared across many
    // lint passes in the same crate. The properties of this config will appear in
    // the root of the config for the crate.
    #[config(section = ".")]
    shared_config: SharedConfig,

    // Some cross-lint-crate configs may be declared like this. Lints that depend on
    // an MSRV would declare an `msrv` field in their lint pass to get info about
    // the configured MSRV. Such config would reside outside of lint-crate-local
    // section in the config file.
    msrv: Msrv
}

// This derives `serde::Deserialize` and also submits metadata about the
// config structure to `inventory` registry.
#[lint_config]
struct Config {
    /// Docs for the config
    tweak_foo: u32,

    /// Docs for the config
    toggle_bar: bool,

    /// Docs for the config
    #[deprecated = "You should use something else instead"]
    deprecated_knob: String,
}

#[lint_config]
struct SharedConfig { /**/ }

impl marker_api::LintPass for LintPass {
    // ...
}
```

When lint crates are compiled they are compiled in an internal cargo workspace that is created by Marker under a `target/marker` directory. This cargo workspace is referenced as "the buildspace". The buildspace contains a single crate with a `cdylib` target that has `[dependencies]` on the 3rd-party lint crates specified in the Marker lints config. This internal "dispatch" crate links the lint crates statically, and uses `inventory::iter` to collect all the lint passes and lints from the included crates and dispatch execution to all of them.

The internal "dispatch" crate will look roughly like this (implementation details are not yet known):

```rust
// All crates must use a single `marker_api` version. If there are multiple incompatible
// `marker_api` crate versions in the dependency tree, then `cargo marker` should find
// that via `cargo metadata` and reject the compilation requiring lint crates to agree
// on a single `marker_api` version compatible with each other and the current driver.
// So there is no need to have a `MARKER_API_VERSION` constant.

// The lint pass that combines all lint passes from all dependent crates and dispatches
// to them
struct LintPass {
    lint_passes: &'static [&'static dyn LintPass]
}

impl marker_api::LintPass for LintPass {}
```

> Implementation thinking 🤔. The code higher might be the expansion of `marker_api::lint_crate! {}`
> to make it consistent between the compilation of lints in the "dispatch" crate and in isolation.
> In this case it may still make sense to have a `MARKER_API_VERSION` static.

## How are lint crate dependencies locked?

Lint declarations in `[lints]` that don't use `path` specification are compiled in an internal cargo workspace with a "dispatch" crate that combines lints from all crates into one dynamic library. Such lint crates appear in `Marker.lock` file. You should commit this file to your source control to get reproducible builds.

Lint crates that are defined with `path` links are compiled separately from all other kinds of lint crate dependencies. They are assumed to be part of the cargo workspace that is linted, and that they already appear in its `Cargo.lock`.

The lint crates and their configuration is specified in a workspace-wide `Marker.toml` file. This would allow to have the lint dependency declaration and its configuration in a single consistent place like this:

```toml
# Marker.toml

[lints]
lint-crate = { path = "./lint-crate" }
marker_lints = "0.4.0"

[lints.marker_lints.config]
config_foo = "bar"
```

I like `Marker.toml` because it also gives `Marker` more visibility. When looking at the repo files this gives you an instant recognition that this project uses Marker. It also makes `Marker.toml` and `Marker.lock` paired just like `Cargo.toml` and `Cargo.lock`.

**Example:**

Suppose you have your own `lint-crate` inside of the workspace that needs to be linted and a dependency on `marker_lints` crate from `crates.io`. The folder structure of such a workspace would look like the following.

- `workspace-crate/`: directory with a regular Rust crate that you develop as part of your app
- `lint-crate/`
    - `Cargo.toml`: declares a dependency on `marker_api`
    - `src/`
        - `lib.rs`: defines lints, configs, implements the lint passes
- `Cargo.toml`: declares `workspace-crate` and `lint-crate` as `workspace.members`
- `Cargo.lock`: contains entries for `lint-crate` and its `marker_api` dependency
- `Marker.toml`: defines `lint-crate` and `marker_lints` under the `[lints]` dependencies table
- `Marker.lock`: hardlink to `target/marker/Cargo.lock` and contains only `marker_lints` and its `marker_api` entries
- `target/`
    - `marker/`
        - `Cargo.toml`: contains a definition of the `buildspace` and the "dispatch" crate. The `[dependencies]` section includes only `marker_lints = "x.y.z"`. The `path` dependencies never get into the `dependencies` here.

### `Marker.lock`

The `Marker.lock` file is a hardlink to the internal buildspace's `Cargo.lock` file. The internal buildspace structure is not stable yet, and not exposed fully to the users, but you may expect it to have a `Cargo.lock` file, and apply any commands that manage the lock file or read the metadata.

The `cargo marker` CLI exposes a command `cargo marker buildspace` that can be suffixed with any `cargo` command and it will be run inside of the internal Marker buildspace to manage the lock file, licensing, do crates pre-fetching for caching etc. For example, you may run commands like the following.

- `cargo marker buildspace update --package foo`
- `cargo marker buildspace deny`
- `cargo marker buildspace tree --depth 1`
- `cargo marker buildspace generate-lockfile`
- `cargo marker buildspace fetch`

The commands that may be run inside of the buildspace this way aren't limited to make possible the usage of external cargo plugins that manage the lockfile. We'll however try to maintain a list of commands that are known to be safe to use in the docs.

# Reference-level explanation

## Buildspace

The `cargo marker` CLI will always try to create a hardlink to `Marker.lock` in the internal buildspace using `cargo generate-lockfile` if it doesn't exist and there are non-`path` dependencies. The users will decide on their own if they want to commit this file to their source control or not.

The build process will first compile the non-`path` lint crates by adding them all into the `dependencies` of the "dispatch" crate, generating code in that "dispatch" crate to define a single `LintPass` implementation that dispatches control flow to all other lint passes from the dependencies.

During the exprimentation, there was found a caveat that if the crate dependency is not referenced in the code, then [`inventory`](https://docs.rs/inventory) crate doesn't return lints from that crate. The assumption is that `cargo` or `rustc` perform dead code elimination and completely exclude the entire crate if it is not used in the calling crate. A workaround for this is to add an explicit `extern crate` declaration per each dependent lint crate during the codegen.

The `path` dependencies will then be compiled in a second step, but they will be compiled in the context of their parent cargo workspace. They will be compiled with a `cargo rustc --crate-type cdylib` and a special `cfg` flag that automatically adds a "dispatch" lint pass to this crate in the `marker_api::lint_crate! {}` macro invocation.

## Why should there be a "dispatch" crate in the buildspace?

I don't like this special case, but this allows us to have a single build invocation for all the 3rd-party lint crates that also produces just one DLL artifact that should supposedly make the compilation faster and produce more optimized code because rustc has static information about all the 3rd-party lint crates when compiling the `cdylib`.

This also keeps the lint crates declared as "static libraries" which works around the inability to run `cargo test --doc` for them ([#303](https://github.com/rust-marker/marker/issues/303)).

We can't go without this "dispatch" crate because the command that allows us to build the crates with the crate type override (`cargo rustc --crate-type cdylib`) works only on a crate-by-crate basis. We would need to make the 3rd-party lint crates members of the buildspace cargo workspace, which isn't possible because they reside in different folders outside of the buildspace inside of the cargo home.

Cargo provides a way to have such a distributed workspace, but only via a [`package.workspace` config](https://doc.rust-lang.org/cargo/reference/manifest.html#the-workspace-field). This means we would need to modify the `[package]` section of the 3rd-party crate in cargo home, which isn't acceptable because cargo home is supposed to be readonly, and the 3rd-party lint crates may be reused across many repositories.

The other approach would be to have a symlink or copy the 3rd-party crates into the buildspace. The symlink will definitely have problems on Windows because creating a symlink on Windows requires elevated user permissions (you basically need to run the terminal as an Administrator or globally switch your Windows into "Developer mode").

I guess all these alternatives aren't better than having a "dispatch" crate.

## Corner case. Renames via `lib.name`

We may be smart enough to allow for `lib.name` renames in the lint crates. We could read that info from the `cargo metadata` while generating the `extern crate` declarations. That should be easy.

# Rationale and alternatives

Having in-workspace lint crates appear in `Marker.lock` would lead to surprising behaviors.

For example, when lints are written developers use "go to definition" on the symbol from `marker_api`. The IDE will open the version of `marker_api` that appears in the workspace's `Cargo.lock` file because it has no knowledge of `Marker.lock`. Same thing happens when in-workspace lint crates are compiled. One would expect such lint crates to use `Cargo.lock` file from the workspace.

The distinction between `path` and non-`path` lint crate dependencies is unfortunate, but this is the least "evil" that was found.

## Alternative. No special case for `path` dependencies

In this case `path` dependencies appear in the `Marker.lock` as well. This would simplify the implementation for us and make the lock file behavior a bit more consistent. But is it worth it? The workspace's `Cargo.lock` file will contain an entry for the in-workspace crate that doesn't actually represent how that crate is built. That may be a potential footgun. I already imagine someone debugging their lint crate, jumping to definition into the `marker_api` in their IDE, reading the code that they think would be compiled, but when they run their lint the `marker_api` version that is actually used isn't the one that they see in their IDE.

The advantage of this approach is consistency. Maybe we can make this work if `cargo marker` validates that the `marker_api` version (if present) in the linted workspace's `Cargo.lock` is equal to the version of `marker_api` in `Marker.lock`. But what if the lint crate uses something more than `marker_api`? We would need to validate the whole dependency tree of the lint crate in the workspace. And what if there are version differences? We should somehow synchronize `Marker.lock` with the workspace's `Cargo.lock`. That's not an easy task.

## Alternative. Compile all lint crates in isolation within their workspaces.

This means we do N distinct `cargo rustc --crate-type cdylib` invocations in the context of each crate's directory. For 3rd-party crates we would run this command in their directory in cargo home using the `Cargo.lock` file that they have there (the one that was published with the crate). This is arguably slower, and each lint crate will have its own patch version of `marker_api` compiled with it.

When the lint crates are included from `crates.io` they are not guaranteed to have a `Cargo.lock` file, and, besides, that lock file isn't easily visible to the users. Tools like `cargo deny / cargo tree` aren't designed to reach out to that `Cargo.lock` file. Besides, no `cargo update` can be done for those lock files, so users are stuck with the versions of the dependencies that those 3rd-party crates have in their lock file.

## Alternative. Not having `marker_api::lint_crate! {}`

This macro adds the "dispatch" lint pass glue to the crate where it is defined and thus makes it possible to compile it in isolation via `cargo rustc --crate-type cdylib`. We could remove such requirement, and instead always compile lint crates with the `marker_rustc_driver` implementation where we could have special handling configured via an env var, for example. That special handling would implicitly add the "dispatch" lint pass to the crate's `lib.rs`.

This won't be easy to implement, but we could look into that in the future.

# Future possibilities

## Pre-compiled lint crates influence

We may support pre-compiled lints in the future. The design for that feature deserves its own RFC, but we may lay out the ground to make that extension possible without fully rewriting the logic of compiling lints from source and handling the lock file for that.

As a starting point, the pre-compiled lint crates don't need a lock file for their dependencies. They are already compiled and they have their dependencies in them, so the only thing we could lock here is the version of the pre-compiled lint crate itself.

For example, when the user requests `lint_crate = { version = "0.3", pre-compiled = true }` then `cargo marker` will try to discover if there is a pre-compiled lint library compatible with the current platform, find its latest patch version and record that in a lock file. But how will we do that if `Marker.lock` is just a hardlink to the buildspace's `Cargo.lock`? We'll likely introduce a separate `Marker.bin.lock` for that. That one will have custom format and will enumerate only the versions of the pre-compiled lint crates. This way it will be possible to have both compiled-from-source and pre-compiled crates dependencies at once.

It may seem it would be better to have a single `Marker.lock` and store both the lock info for the crates compiled from source and pre-compiled lint crates dependencies, but this will probably make the implementation more complex, and will make it a bit harder to navigate. Anyway, we'll revisit this in the future once we actually start working on pre-compiled lints support.
