This document describes the release processes and artifacts in `marker`.

# Artifacts

As a result of the engineering effort invested into `marker` we get the following items. Each of them comprises the public API of this project.

## Source packages in `crates.io`

There are library packages published to crates.io such as `marker_api`, `marker_uitest`. They are intended to be statically linked to the users' lint crates.

There are binary packages published to crates.io such as `cargo_marker`, `marker_rustc_driver`. They are intended to be used only in exceptional cases to let users build binaries from source. For example, if `marker` lacks support for some platform where users want to utilize it, then they may leverage `cargo install` to compile `marker`'s component binaries from source. However, 99% of the time users will want to download a precompiled binary.

## Pre-compiled binaries published in a GitHub release

These are the archived binaries such as `cargo-marker-x86_64-unknown-linux-gnu.tar.gz`. These binaries cover as much platforms as possible meaning different combinations of OS (`windows`, `linux`, `macos`), arch (`x86_64`, `aarch64`), libc implementations (`gnu`, `musl`) and their different versions.

Let's review what artifacts there are for a single platform. Take this list as an example.

- `cargo-marker-aarch64-unknown-linux-gnu.tar.gz`
- `cargo-marker-aarch64-unknown-linux-gnu.zip`
- `cargo-marker-aarch64-unknown-linux-gnu.sha256`

The `tar.gz` and `zip` archives contain a single `cargo-marker` binary file in them. It doesn't matter what archive the user downloads. They have the choice to select either of them. It is usually common to have only `tar.gz` format for `unix`-like systems, and `zip` for `windows`, but we provide both for the reason described below.

For the cross-platform code it is much more convenient when the same archive format is available for every platform consistently. This means, for example, `cargo-marker` may depend only on the `zip` crate exclusively and expect that `zip` of the  `marker_rustc_driver` is always available for download on all platforms.

> Here is a case of [`terraform`'s](https://developer.hashicorp.com/terraform/downloads) binaries. They are distributed in `zip` for all platforms.

GitHub is very generous with the releases storage limits. They are [almost unlimited](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases#storage-and-bandwidth-quotas). They difinitely want your project to be open-source 😉.

The `sha256` sum is a small file that users may optionally download together with the archive itself to verify the integrity of the archive. It serves as a signature of the artifact to make sure it was downloaded as expected bit-by-bit with what was published effectively detecting corruptions during the download and making it harder to forge artifacts for malicious actors.

<!-- region replace-version stable -->
This [`install.sh`](https://raw.githubusercontent.com/rust-marker/marker/v0.3.0/scripts/release/install.sh) script, can be used to automatically download and verify Marker's binaries.
<!-- endregion replace-version stable -->

### Operating system versions coverage

We want to cover not only the mainstream operating systems, but also some reasonable range of their versions. For example, for `ubuntu` we want our binaries to work on the current LTS version of `ubuntu` and also on the version of `ubuntu` that precedes it. At the time of this writing the latest LTS version of `ubuntu` is `22.04`, and the previous one is `20.04`, while the LTS version of ubuntu before that one is `18.04` and it already reached the "end of standard support" date, and so far practically noone uses it.

The platforms that we use for builds are pinned to specific OS versions. It gives us confidence that our binaries will run on these platforms or more modern ones. Operating systems are usually backward-compatible, but they are not forward-compatible.

It means, you can't compile a binary on `ubuntu-22.04` and run it on `ubuntu-20.04` because the GLIBC that the binary will require will be too high for `ubuntu-20.04`. The version of GLIBC installed on `ubuntu-20.04` is `2.29`, and the version of GLIBC installed on `ubuntu-22.04` is `2.34`. This is based on the experience of the problem that appeared in [nushell](https://github.com/nushell/nushell/issues/7282) project.

> The story of `nushell` was the following.
>
> They used `ubuntu-latest` to build their pre-compiled binaries. At that moment `ubuntu-latest` was aliased to `20.04` and the whole world was sitting on this version of `ubuntu` and everything was fine.
>
> But then some day `ubuntu-22.04` became generally available on GitHub runners and `ubuntu-latest` was changed to point to this new version of `ubuntu`. When the new release of `nushell` was created they built it on `ubuntu-latest` a.k.a. `ubuntu-22.04` at that point in time.
>
> The whole world was still sitting on `ubuntu-20.04` and lazily migrating to `ubuntu-22.04` so, consequently, there were multiple users who wanted to upgrade `nushell` but they were still sitting on `ubuntu-20.04`. The binaries from the new `nushell` release were not working for them due to the higher GLIBC requirement.

You can check for yourself the GLIBC version requirement of your binary with the following command.

```bash
objdump -T ./path/to/binary | grep -Eo 'GLIBC_[^)]+' | sort -V | tail -1
```

### Universal `musl` binaries

The binaries with `musl` in their name don't depend on GLIBC. They are statically linked meaning they have no dynamic library dependencies. This way they support a wide range of `linux` distributions meaning that you may expect them to run almost on any `linux` distro of any version.

This comes at a cost, though. The `musl` implementation of `libc` isn't complete, and it may have bugs, performance degradations compared to GLIBC, etc. However, it generally covers everything you may ever need if you aren't doing something unusual.

## Github Action

There is a Github Action template `action.yml` file maintained within the Marker repository. It installs the pre-compiled binaries using the installation scripts and runs `cargo marker`.

It is common to reference the Github Action using sliding tags like this:
```yml
- uses: rust-marker/marker@v0.3
```
Or this when marker reaches a `1.0.0` version and minor versions are compatible:
```yml
- uses: rust-marker/marker@v1
```

We maintain these tags in our automated release flow described below.

# Regular release

The regular release means a planned event when the maintainer of `marker` publishes the new version of the artifacts to the users. It may happen at any time when such a decision is made, which usually means on some consistent schedule.

The release process is semi-automated, and thus requires the human involvement.

The maintainer has to decide what the next release semver version number will be and prepare the `CHANGELOG.md` file with the description of the new release. The description should follow a consistent structure [like this](https://keepachangelog.com/en/1.0.0/).

The new versions are always prepended to the top of the changelog file. The numbered version at the top is always considered to be the latest release of `marker`. Before invoking the release automation a human must make sure that a new entry with the new version number was created in the `CHANGELOG.md` file.

This flow also allows for pre-releases. These are the ones that contain a `-suffix` in their name e.g. `1.0.0-rc` or even `1.0.0-rc.2` etc. You just need to prepend an entry with this version to the changelog.

## `release` workflow

Once the `CHANGELOG.md` is ready in `master` the maintainer can trigger the CI `release` workflow from GitHub "Actions" web page. That workflow contains no input variables to set the version. It fully relies on the `CHANGELOG.md` as an input for that.

The `release` CI workflow then checks out the `master` branch (assuming "Use workflow from" input wasn't changed from its default), parses the current release version from the `CHANGELOG.md` file and updates `Cargo.toml`, `Cargo.lock` and various `.rs` and `.md` files with the new version. It uses simple regex patterns with `sed` to edit the files. This logic may break, and thus there is a test on regular CI that makes sure it stays stable.

The release commit is then assigned a `v{semver}` git tag . After that the workflow sets the next version for the new development cycle with the incremented release version and the `-dev` suffix, and creates a new commit with that.

### Sliding `v{major}` and `v{major}.{minor}` tags.

The release CI flow will also create and move the sliding `v{major}` and `v{major}.{minor}` tags. For example, if we release a version `0.3.0`, then there will be three tags created `v0.3.0`, `v0` and `v0.3`.

And if we release a patch `0.3.1`, then there will be a new tag `v0.3.1`, and the existing `v0` and `v0.3` will be moved to this new  commit for `0.3.1`.

The sliding tags won't be updated for pre-releases (versions with `-` suffix in them).

These sliding tags may be used to hardcode only the major or minor version of `marker`, to allow for automatic updates, to the patch version. These sliding tags can be used to download the installation script or run Marker's Github Action and automatically get fixes from the upstream.

## `release-on-tag` workflow

The other CI workflow called `release-on-tag` triggers on the new `v{semver}` git tag in the repository. It does the following.

1. It creates a new GitHub release associated with the new tag. The description of the GitHub release will contain a copy of the section corresponding to the new version from the `CHANGELOG.md`.
2. It builds the pre-compiled binaries for multiple platforms and uploads them to the new GitHub release.
3. It publishes the new version of packages to `crates.io`.

Once this workflow finishes successfully, the release process may be considered complete.

# Hotfix backport release

There may be a case that a new critical bug was revealed in the released version of `marker` artifacts. It is most likely that bugs come from the new version of code, so you'll probably want to fix the bug in the current version of code present in `master`, increment the patch version and release it. The release flow in this case is identical to the [regular release](#regular-release) with the specificity that the maintainer is expected to make a patch bump in the changelog.

A more elaborate case is when the bug was fixed in the latest version of `marker` but there still are users who sit on the previous incompatible version of `marker`, and upgrading to the latest one for them may take too much effort and time, but they desperately need a fix today.

For this case you need to check out the older version of `marker` in a new git branch. Use the git tags to find the proper commit for that. Make the fix in your branch and call it `hotfix/X.Y.Z`, for example. The format of the branch name doesn't matter, but let's use this one just for consistency.

Push the branch to the upstream `marker` repository. You don't want to merge that branch into `master`, because master is already far ahead of that branch and contains the fix in the new code. You just need to create it in the upstream so that the CI workflows in that repo can use it.

Add a new entry with the new patch to the `CHANGELOG.md` file in that branch just like during the regular release. Now trigger the `release` CI workflow, but specify your custom hotfix branch in the "Use workflow from" input selector. Once that is done you should also cherry-pick the commit that updated the `CHANGELOG.md` into the `master` branch for the history. This backported fix should still appear at the top of the changelog file, but, of course the next release's version should still be the increment of the previous version within the semver compatibility range set for the current development cycle.
