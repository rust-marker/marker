# CI

Marker's primary objective is to offer an excellent linting interface, including the seamless integration with CI services. This document outlines the available CI tooling and provides example templates.

<!-- toc -->

## GitHub Action

Marker provides a GitHub Action that downloads the pre-compiled binaries and runs `cargo marker`.

<!-- region replace marker version stable -->

```yml
- uses: rust-marker/marker@v0.5
```

### Git tags

The git tag specified in the GitHub Action indicates which version of Marker should be installed. There are several tag flavors available:

- **Sliding tags, like `v0.5` *(recommended)*:**

  Use this to get automatic patch updates.

- **Fixed tags, like `v0.5.0`:**

  Use this to pin a specific patch version. If you find a regression in a patch version, please create a [new issue]. Patch versions must never break anything!

<!-- endregion replace marker version stable -->

> ⚠️ The minor versions before Marker `v1` contain breaking changes. While there is a sliding `v0` tag, it's highly recommended to include the minor version as well. This prevents uncontrolled CI breakage with every release.

### Inputs

All inputs are optional, they only allow tweaking the default behavior.

| Name           | Description                                                   | Type      | Default |
|----------------|---------------------------------------------------------------|-----------|---------|
| `install-only` | Only install Marker but don't run the `cargo marker` command. | `boolean` | `false` |


### Environment variables

| Name                         | Description                                                              | Type      | Default |
|------------------------------|--------------------------------------------------------------------------|-----------|---------|
| `MARKER_NET_RETRY_COUNT`     | Max number of retries for downloads. This also sets `RUSTUP_MAX_RETRIES` | `integer` | `5`     |
| `MARKER_NET_RETRY_MAX_DELAY` | Max delay between subsequent retries for downloads in seconds            | `integer` | `60`    |

These environment variables configure the behavior of the installation script and they may be used if you run that script directly as well e.g. on [other CI systems](#other-ci-systems).

### Example workflows

These example workflows will use the lint crates specified in the `Cargo.toml` file by default. Refer to the [*Lint Crate Declaration*](./lint-crate-declaration.md) section for more information.

#### Basic usage

Checkout the repository code, install the toolchain, Marker, and start linting.

<!-- region replace marker action version stable -->
```yml
jobs:
  rust-marker-lints:
    runs-on: ubuntu-latest
    env:
      RUSTFLAGS: --deny warnings
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - uses: rust-marker/marker@v0.5
```
<!-- endregion replace marker action version stable -->


#### Advanced usage

If you need something more than just the `cargo marker` command, you may use the action to only install Marker and then manually run the `cargo marker` command, just like in your local dev environment.

Here is an example of how you could limit the set of crates that you want to lint. Refer to `cargo marker --help` for a full list of available options.

<!-- region replace marker action version stable -->
```yml
jobs:
  rust-marker-lints:
    runs-on: ubuntu-latest
    env:
      RUSTFLAGS: --deny warnings
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - uses: rust-marker/marker@v0.5
        with:
          install-only: true
      - run: cargo marker -- -p crate-foo -p crate-bar
```
<!-- endregion replace marker action version stable -->

If you have an example of advanced usage of `cargo marker` command that you have to repeat in your CI template again and again consider opening a [new issue] in our repository. We will be glad to hear any suggestions about extending the inputs for the GitHub Action for your use case.


### GitHub-managed runners

The action is cross-platform. It supports Windows, Linux and MacOS. It is tested on all [OS images supported by managed GitHub Actions runners].

If GitHub releases a newer OS image version it is very likely that this action will still "just work" on it. We make sure to add new OS images that GitHub-managed runners support to our CI test suite, as well as remove support for the old ones following the GitHub Actions's OS images deprecation cadance.

### Self-hosted runners

The action must work on self-hosted runners out of the box as well. We don't test all possible operating systems on our CI, but if your self-hosted runners use the OS version from the range of [OS images supported by managed GitHub Actions runners] or other compatible distributions, then it should work. If you have some exotic OS setup which breaks our GitHub Action we will be interested to hear about that in a [new issue].

## Other CI systems

If you don't use GitHub Actions CI, you can still benefit from the pre-made installation scripts that automate the downloading of the pre-compiled binaries on CI for you.

These curl commands differ slightly from the scripts mentioned in the [installation chapter](installation.md#download-pre-compiled-binaries-recommended). They are more verbose for additional readability in the CI templates, and they also contain additional options to retry spurious network errors for stability on CI.

You can run these scripts on any CI system of your choice, and they will make the `cargo marker` command available for you.

<!-- region replace marker version stable -->

**Linux or MacOS (Bash)**:
```bash
curl \
    --location \
    --silent \
    --fail \
    --show-error \
    --retry 5 \
    --retry-all-errors \
    https://raw.githubusercontent.com/rust-marker/marker/v0.5/scripts/release/install.sh \
    | bash
```

**Windows (PowerShell)**:
```ps1
curl.exe `
    --location `
    --silent `
    --fail `
    --show-error `
    --retry 5 `
    --retry-all-errors `
    https://raw.githubusercontent.com/rust-marker/marker/v0.5/scripts/release/install.ps1 `
    | powershell -command -
```

<!-- endregion replace marker version stable -->

Both of these scripts are configurable. See the [environment variables](#environment-variables) for details on what's available.

The available version git tags that you may use in the URL are described in the [git tags](#git-tags) paragraph of the Github Action.

[`RUSTUP_MAX_RETRIES`]: https://github.com/rust-lang/rustup/blob/5af4bc4a0d4bc69ea9091a7935fb3783c5fb508e/doc/dev-guide/src/tips-and-tricks.md#rustup_max_retries
[new issue]: https://gitHub.com/rust-marker/marker/issues/new/choose
[OS images supported by managed GitHub Actions runners]: https://docs.github.com/en/actions/using-github-hosted-runners/about-github-hosted-runners/about-github-hosted-runners#supported-runners-and-hardware-resources
