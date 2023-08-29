[Marker]: https://github.com/rust-marker/marker
[Marker's Readme]: https://github.com/rust-marker/marker/blob/master/README.md
[`EnvFilter`]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html
[`miette`]: https://docs.rs/miette
[`thiserror`]: https://docs.rs/thiserror

[![Crates.io](https://img.shields.io/crates/v/marker_error.svg)](https://crates.io/crates/marker_error)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/marker_error.svg)](#license)

# Marker Error 🔴

This crate provides a common error handling utilities for all [Marker] crates. You're welcome to check out [Marker's Readme] if you're interested in the project.

> **Warning**
>
> This crate is not part of Marker's official API, it's only intended to be used by marker internally.

The goal of error handling is to provide diagnostic information about the context of the error for both the user and the developers of `marker`. This means there are at least two actors here that need to observe the errors.

# User's perspective

## Understanding simple mistakes

Users want to see a succinct but informative error message in case they do something wrong. The error message must direct the users to the reason for the failure and optionally suggest ways to fix it.

### Solution

It is enough to print the error message into `stderr` of the process. The users will be able to see it in their terminal or CI logs. The error should not contain any verbose diagnostic information like backtraces or spantraces by default.

Use `Result` type for reporting such errors. The crate that is responsible for rendering this type of errors was selected to be [`miette`]. This crate renders the chain of errors that happened during the execution of `marker` code in a readable colorful way. It also allows including the snippets of text that the error refers to; for example, if the config is invalid, we may show the snippet of the config that is invalid and describe what part of it is wrong.

[`miette`] builds on top of [`thiserror`] and extends the `std::error::Error` with additional functionality like the one mentioned above, error codes, help messages, multiple error handling, etc. This crate propagates using both [`miette`] and [`thiserror`] in combination to achieve the goal of user-friendly error reporting.

#### Error chains

The errors have the concept of a "chain". There is a source error that was the original cause of the problem, and all other errors that wrap the source one during the propagation of the error through the call stack. Make sure to never include the `source` error's message in the message of the error that wraps it. This is because [`miette`] will still render the error messages for all errors in the chain. Here is an example of a problematic code.

```rust
#[derive(Debug, thiserror::Error)]
enum RootError {
    #[error("Oh no, Foo happened: {0}")]
    Foo(FooError)
}

#[derive(Debug, thiserror::Error)]
#[error("Foo failed with factor {foo_factor}")]
struct FooError {
    foo_factor: usize
}
```

In this case, when the `RootError` is output you will see something like the following.

```log
  × Oh no, Foo happened: Foo failed with factor 42
  ╰─▶ Foo failed with factor 42
```

You may notice the repetition of messages here.

If you remove the `: {0}` from the error message of `RootError` the error output will be less redundant.

```log
  × Oh no, Foo happened
  ╰─▶ Foo failed with factor 42
```

## Developing workarounds for bugs

In case the user stumbles with a bug in `marker` they need to understand this is indeed a bug. Usually, when the error is caused by the user, they can figure this out. If the error is caused by a bug in `marker` the users may want to dig a bit deeper into the internals of `marker` code and find out if there is any workaround they could apply to overcome the problem to unblock them for the time being.

### Solution

It's possible to request additional diagnostic logging output by leveraging the `MARKER_ERROR_TRACE=1` optionally in combination with the `MARKER_LOG` environment variable. The variable `MARKER_ERROR_TRACE=1` is used to enable the capturing of the spantrace and backtrace at the point where the error happened. The variable `MARKER_LOG` is used to configure the [`EnvFilter`] for the `tracing` logging framework enabling both more verbose logging and spantrace capturing.

## Reporting errors to `marker` developers

If the user thinks that the error is a bug in `marker` code, they should be able to report that error as a GitHub issue. It should be convenient for users to collect the diagnostic info about the context of the bug and send it to us.

### Solution

The users may capture the output of the `cargo-marker` command that they ran by redirecting it to a file, or by copying the logs from their terminal if there are not too many of them. Ideally, `cargo-marker` should dump the diagnostic info automatically or with a separate subcommand to collect as much relevant info as possible. See also the [historical trace paragraph](#get-historical-trace-of-the-application) about that.

# `marker` developer's perspective

## Reporting bugs

It should be simple to report bugs in the code. The result of the bug should be observable. If the bug is critical, then the process will most likely be shut down, but it should also be possible to produce warning-level events if the impaired subsystem doesn't obstruct the execution of the program.

### Solution

This is the same as [reporting errors to marker developers](#reporting-errors-to-marker-developers), but in case of bugs we collect the diagnostic info automatically, with as much user interaction as possible. For example, we collect the spantrace and backtrace in case of a panic regardless of the `RUST_BACKTRACE` env var. For that we have a custom panic hook in `cargo-marker` that does this.

## Reporting user-recoverable errors

Some errors can be caused by the users themselves. For example, the user wrote a config in TOML format with some invalid data in it like a non-existing lint crate was specified. Such error is expected to happen and isn't a bug in `marker` code.

### Solution

This crate provides be an easy way to report such errors to the users that is distinct from reporting bugs, which doesn't flood the users with tons of diagnostic information if such error happens.

This creat requires as minimum actions as possible to handle a user-recoverable error. It is custom to use enums of errors, but people often over-invest in this. Enums are usually used when structural error handling is required, where a different action may be taken to remediate some special kind of error, and the calling code wants to take that apart from all other errors.

Most of the errors in `cargo-marker` are just propagated to the caller such that no code in `cargo-marker` matches on them. Such errors are easy to create directly in the code that produces them. There isn't an obligation to create a new enum variant for a new type of error. The error variants may be created if there is actually a demand for matching on a specific kind of error.

As a prior art you may to take a look at `cargo`'s codebase. They've been using `anyhow` for all their error handling and it's been probably working fine for them. Subjectively, they could've overused `anyhow`, as there are some places where they had to `downcast_ref` the errors to do structural error handling. Therefore, this crate uses a combined approach where we still have the enum of errors, but that enum has special variants for uncategorized errors which are just propagated to the user. If the error needs to be specially handled in our code we may create a new variant for the special case at any time.

## Reviewing issues in GitHub

When users discover bugs in `marker` the issues that they write should contain enough diagnostic information about the context of the bug. There should be ways for users to collect as much diagnostic info as possible even in case the error that is returned to them is a recoverable one. In such case, we would like to have an ability to force our error reporting mechanism to collect the same volume of information as the one collected when a bug is detected.

### Solution

With `MARKER_LOG` env variable we may ask our users to rerun `cargo-marker` with higher levels of logging. This will also capture spantrace and backtrace for errors that they reported. Ideally, users could attach a historical log of their operations to us generated by `cargo-marker` (see below).

## Get historical trace of the application

A really rare and annoying class of errors is the errors that aren't easily reproducible. For example, the case when the user has a toolchain version drift or a special environment variable that is present only on their machines that somehow breaks our logic. Such "flaky" errors that reproduce only on one machine and don't reproduce on the other one are just part of this class. There are also other potential bugs in the process scheduling and their order of execution which just can't be reproduced in a stable way.

For such cases, it's best to have a historical log of the operations that were performed by the user with the full volume of all telemetry collected during them. This historical log is meant to be consumed by `marker` developers only to restore the chain of actions that made `marker` end up in the state when the error happened. This way we could debug this situation by looking at the historical log only even if we can't reproduce the problem on our side.

It is important that the historical log must contain the trace from independent runs of `marker`. It means it should be stored persistently across the process runs. For example, if the user installs the toolchain and then runs `marker` linting we would like to see both of these actions in the history.

### Solution (not implemented yet)

> ⚠️ This section describes the feature that is not implemented yet as it is not very critical. We may or may not implement it at some point in the future.

`cargo-marker` may store log files in some known location where it records all logs from its execution. The log files should be rotated and limited in size to prevent running out of disk space. Then users should be able to send us the log files by attaching them in a GitHub issue. There may be a subcommand in `cargo-marker` that dumps an archive of the log files making it easier for users to attach the logs.


## Contributing

Contributions are highly appreciated! If you encounter any issues or have suggestions for improvements, please check out [Marker's GitHub repository](https://github.com/rust-marker/marker).

## License

Copyright (c) 2022-2023 Rust-Marker

Rust-marker is distributed under the terms of the MIT license or the Apache License (Version 2.0).

See [LICENSE-APACHE](https://github.com/rust-marker/marker/blob/master/LICENSE-APACHE), [LICENSE-MIT](https://github.com/rust-marker/marker/blob/master/LICENSE-MIT).
