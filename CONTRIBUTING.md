# Contributing to Marker

Thank you for your interest in contributing to Marker. All contributions are appreciated!

## Table of Content

* [Table of Content](#table-of-content)
* [Code of Conduct](#code-of-conduct)
* [Ways to Contribute](#ways-to-contribute)
    * [Create an Issue](#create-an-issue)
    * [Code contributions](#code-contributions)
        * [Select a Task](#select-a-task)
        * [Fork and Clone Marker](#fork-and-clone-marker)
        * [Rust-Analyzer Setup](#rust-analyzer-setup)
        * [Change Marker](#change-marker)
        * [Test Marker](#test-marker)
        * [Submit a Pull Request](#submit-a-pull-request)
    * [Test Marker](#test-marker-1)
    * [Review Pull Requests](#review-pull-requests)
* [Style Guide](#style-guide)
    * [Rebase Workflow](#rebase-workflow)
    * [Commit Messages](#commit-messages)
    * [Formatting](#formatting)
    * [Jokes](#jokes)
* [Contact](#contact)
* [Legal Notice](#legal-notice)

## Code of Conduct

This project follows [Rust's Code of Conduct](https://www.rust-lang.org/conduct.html). See [Marker's *Code of Conduct* file](./CODE_OF_CONDUCT.md) for more information.

## Ways to Contribute

### Create an Issue

Issues are the right place to start if you've found a bug, want to suggest a feature, or request help. Before you create a new issue, it would be a big help if you checked the existing issues to see if the bug, feature, or question already exists.

* **Bug Reports**

    If you find a bug, please try to include as much information as possible. It would be perfect if the following points were addressed:
    * The bug you found
    * An example how it can be reproduced
    * The faulty output from Marker
    * The version of Marker you're using
    * The toolchain you're using

* **Feature Requests**

    If you want to suggest a feature, please also explain what you want to archive with it. It would be perfect if the following information were included:
    * The feature description
    * The motivation behind this feature
    * Which component should provide the feature
    * Any additional information that can be useful for development

* **Questions**

    If you have a question, please always ask. In your question, it would be good if you included the following information, if applicable:
    * Your question
    * The version of Marker you're using
    * The component of Marker you're asking about

### Code contributions

Contributing code to Marker is a great way to help. The following is a quick overview of the steps involved in contributing code.

#### Select a Task

If you're looking for something to work on, please check out Marker's [open issues]. You can also filter for the [`E-good-first-issue` label] to find easier issues that give a gentle introduction to Marker's code.

You can also work on tasks not yet tracked in the [open issue]. For this, please refer to the [*Create an Issue*](#create-an-issue) section. Generally speaking, bug fixes, spelling corrections, or new utilities are always welcome. Changes to the public interface should first be discussed in an issue to ensure that the update will later be accepted.

Once you've decided on an issue, please comment on it and say that you want to work on it. You can also ping any active maintainer to receive mentoring instructions.

[open issues]: https://github.com/rust-marker/marker/issues
[`E-good-first-issue` label]: https://github.com/rust-marker/marker/labels/E-good-first-issue

#### Fork and Clone Marker

To get started, you have to fork Marker and clone the repository. The following is a quick overview of the involved steps:
1. Fork the project on GitHub.
2. Clone the repository into your preferred dev environment.
3. Add the `rust-marker/marker` repository as an upstream remote:
    ```sh
    git remote add upstream git@github.com:rust-marker/marker.git
    ```
4. Run `cargo check` to install the required toolchain and ensure that everything is set up correctly.

That's it. Now you're ready to drive into your first change.

#### Rust-Analyzer Setup

It's recommended to set the [`rust-analyzer.rustc.source`] configuration, to allow autocompletion for rustc types. Rust-Analyzer needs to be restarted, for the change to take affect:

```json
{
    "rust-analyzer.rustc.source": "discover"
}
```

While working on the API, it can also be helpful to enable inlay type hints for elided lifetimes in function signatures. See [`rust-analyzer.inlayHints.lifetimeElisionHints.enable`]:

```json
{
    "rust-analyzer.inlayHints.lifetimeElisionHints.enable": "skip_trivial",
}
```

[`rust-analyzer.rustc.source`]: https://rust-analyzer.github.io/manual.html#rust-analyzer.rustc.source
[`rust-analyzer.inlayHints.lifetimeElisionHints.enable`]: https://rust-analyzer.github.io/manual.html#rust-analyzer.inlayHints.lifetimeElisionHints.enable


#### Change Marker

The type of change and recommended documentation to read depends on the component you want to work on. Here is a collection of useful links. Note that most of these are not targeted towards Marker, but they're still super useful:

* `cargo_marker`: This component uses several well-known dependencies with awesome documentation. Additionally, it has doc comments in most modules.
    * [The Cargo Book](https://doc.rust-lang.org/cargo/)
        * [Environment Variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
    * [The rustup book](https://rust-lang.github.io/rustup/)
    * [clap](https://docs.rs/clap/latest/clap/)
    * [Serde](https://serde.rs/)
* `marker_api`: The AST representation of Marker and everything needed to create a lint crate
    * [The Rust Reference](https://doc.rust-lang.org/reference/introduction.html)
* `marker_rustc_driver`: This component translates rustc's representation to Marker's API.
    * [rustc-dev-guide](https://rustc-dev-guide.rust-lang.org/)
        * [The HIR](https://rustc-dev-guide.rust-lang.org/hir.html)
        * [Identifiers in the compiler](https://rustc-dev-guide.rust-lang.org/identifiers.html)
        * [The ty module: representing types](https://rustc-dev-guide.rust-lang.org/ty.html)
    * [rustc docs](https://doc.rust-lang.org/stable/nightly-rustc/)
        * [`rustc_hir` create](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_hir/index.html)
        * [`rustc_middle::ty::TyCtxt`](https://doc.rust-lang.org/nightly/nightly-rustc/rustc_middle/ty/struct.TyCtxt.html)
* `marker_uitest`: This component is a small wrapper around the ui_test crate
    * [`ui_test` crate documentation](https://docs.rs/ui_test/latest/ui_test/)
* `marker_lints`: This crate provides lints for Marker. It only uses the public interface of Marker:
    * [`marker_api` crate documentation](https://docs.rs/marker_api/latest/marker_api/)
    * [`marker_utils` crate documentation](https://docs.rs/marker_utils/latest/marker_utils/)

Marker is currently missing documentation on how to best approach issues in other components. You're always welcome to reach out and ask for advice. You can also search the code base and reference existing code.

If you want to add a new dependency to any component in Marker, please first ask in the issue if the dependency in question will be accepted.

Please ensure that you've also read Marker's [*Style Guide*](#style-guide).

#### Test Marker

Marker is mostly tested by printing the AST notes. Please check out the `marker_uilints` crate for more information on how:
* [`marker_uilints`](./marker_uilints/README.md)

#### Submit a Pull Request

Once you believe that your update is ready, please create a pull request in the [rust-marker/marker] repository. All relevant information should be included in your PR description. A maintainer will assign themselves to your PR. If you haven't gotten a response within a week, please ping someone from the team.

Every PR is tested to ensure that all tests pass, the documentation can be built, everything is formatted, and relevant crates can be built on the stable toolchain. If any CI check fails, please try to fix it. The log will usually contain information on how to best approach the problem. You can also ask for help in the pull request.

The CI only runs on Ubuntu. Only the final merge is run on all supported platforms to save CI time. If your change should be tested on all platforms early, you can ping a maintainer to kick off a test run.

[rust-marker/marker]: https://github.com/rust-marker/marker

### Test Marker

The current versions of Marker are intended for testing. All feedback is appreciated, ranging from the first impression to a report of what you like and dislike. Marker relies on feedback to improve. The goal is to create the best linting tool that feels good and welcoming to everyone, and this can't be done without the support of the community.

The following is a small collection of ways you can test Marker right now:

1. **Run Marker**

    Simply run Marker on any Rust project you can find and report bugs or unexpected behavior. Once you've installed Marker, you can use the following command to run Marker with the `marker_lints` lint crate:

<!-- region replace-version stable -->
    ```sh
    cargo marker --lints "marker_lints = '0.3.0'"
    ```
<!-- endregion replace-version stable -->

    If you find any bugs or unexpected behavior, please [create an issue]. [rust-marker/marker#198] is a collection of all crates that were linted successfully. You can also add your own crates to the ever-growing list by commenting on the issue.

2. **Write a Lint**

    Marker is an interface for creating lints. The linting API is an essential part of Marker that should feel natural and clear. If you have the time, please just try to create a lint. Write down what features were hard to find or are just missing completely right now.

    If you need a lint idea, you can potentially pick a [user story] for Marker or create a lint based on an existing [Clippy lint].

    It would also be a big help if you could add the lint to the [example lints] repository.

3. **Fuzz Marker**

    All public interfaces of Marker should be resilient and, in the worst case, terminate with a helpful error message. One way to test Marker is to just call every function in the public interface with every input you can think of.

    If you find any bugs or unexpected behavior, please [create an issue].

[rust-marker/marker#198]: https://github.com/rust-marker/marker/issues/198
[create an issue]: https://github.com/rust-marker/marker/issues/new/choose
[example lints]: https://github.com/rust-marker/marker-example-lints
[user story]: https://github.com/rust-marker/design/issues?q=is%3Aissue+is%3Aopen+label%3AA-user-story
[Clippy lint]: https://rust-lang.github.io/rust-clippy/master/index.html

### Review Pull Requests

Software engineering and open-source are team sports. Pull requests are intended to collect feedback and discuss changes before they're added to the master branch. Any feedback or suggestions are valued and treated equally. Please don't hesitate to involve yourself in discussions on pull requests.

## Style Guide

### Rebase Workflow

Marker uses a *Rebase Workflow*, meaning that all branches, besides the `master` branch, should be updated using a rebase. This workflow makes the git history linear and easier to follow. It also aligns Marker with other Rust projects.

This policy means that PRs with merge commits will generally not be merged. Instead, you'll be asked to remove the merge commits and update your branch with a rebase. You can still create a PR to get early feedback or support.

You can also check out rustc's documentation about rebasing and this workflow:
* [No-Merge Policy](https://rustc-dev-guide.rust-lang.org/git.html#no-merge-policy)
* [Rebasing and Conflicts](https://rustc-dev-guide.rust-lang.org/git.html#rebasing-and-conflicts)

### Commit Messages

To better track what component of Marker has been affected by a commit, it's recommended to start the commit message with the name of the component or area. As an example, if you update an expression in the API, the message could be ``API: Update `xyz` expression``, clearly indicating that the commit targets the API. The following prefixes are commonly used:
* `API`: Changes targeting the base API (the `marker_api` crate)
* `utils`: Changes targeting additional utilities for the API (the `marker_utils` crate)
* `cargo`: Changes targeting Cargo's CLI (the `cargo_marker` crate)
* `adapter`: Changes targeting the adapter for drivers (the `marker_adapter` crate)
* `rustc`: Changes targeting rustc's driver (the `marker_rustc_driver` crate)
* `uitest`: Changes to Marker's ui-test setup (the `marker_uitest` crate)
* `lints`: Changes to lints for Marker (the `marker_lints` crate)
* `uilints`: Changes to Marker's uilints for testing (the `marker_uilints` crate)
* `Doc`: Any documentation updates
* `CI`: Any CI updates
* `Chore`: General chores that target basically everything, like releases and nightly bumps

It's generally also recommended to split changes based on the area. If you, for instance, add a new expression to Marker, you can first commit the API changes (including doc comments) and then add the rustc backend and tests in a second commit. For example, adding the `cool` expression might have this history:

1. ``API: Add new `cool` expression to the API``
2. ``Rustc: Backend for the `cool` expression``

Changes affecting multiple components can either just name the main area or list multiple:

* Like this: ``API: Add `MarkerContext::something` function``
* Or this: ``API, adapter, rustc: Add `MarkerContext::something` function``

### Formatting

Everything in Marker should be formatted with rustfmt, this is also actively checked in the CI. You can format your code with the following command:

```
cargo fmt
```

rustfmt currently struggles with [if-let chains](https://rust-lang.github.io/rfcs/2497-if-let-chains.html). As a result, functions with these are not automatically formatted. If you use them, please make sure to manually format the function as best as you can.

### Jokes

Contributing should be fun. Public, user-facing documentation for Marker should be written in a serious manner. In internal functions or UI tests, it's fine to include jokes, puns, or leave some comments about the journey that led you to this magnificent yet horrifying hack that still somehow works.

The repository already contains some small easter eggs, admittedly in the weird humor of @xFrednet, but the point still stands. Let's develop Marker together and have fun at the same time. :D

## Contact

If you have a question or need assistance with your contribution, **don't hesitate to ask**! You're always welcome to [create an issue] or a PR. Using a public channel allows future contributors to also find the information if they get stuck.

You can also contact me, @xFrednet, directly:
* @xFrednet on [rust-lang's Zulip](https://rust-lang.zulipchat.com)
* @xFrednet on Discord
* Or per e-mail: [xFrednet+marker@gmail.com](mailto:xFrednet+marker@gmail.com)

[create an issue]: https://github.com/rust-marker/marker/issues/new

## Legal Notice

Rust-marker is distributed under the terms of the MIT or Apache License (Version 2.0). All contributions fall under these licenses, and as such, it's expected that you authored 100% of the content you contribute and that you have the necessary rights to the content.
