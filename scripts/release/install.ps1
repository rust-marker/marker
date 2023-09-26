#!/usr/bin/env pwsh
#
# This script downloads the `cargo-marker` and the `marker_rustc_driver`
# binaries from the GitHub release assets. It also sets up the required
# Rust toolchain that the `marker_rustc_driver` depends on.
#
# This script must be self-contained! Nothing here should use anything from
# the marker repository, because users are expected to run this script on
# their machines where they don't have the marker repository cloned.
#
# It's possible to run this script on Linux if you have PowerShell installed there.
# An important caveat is that you'll likely install the latest PowerShell 7.0 on linux,
# but on real Windows 10/11 the version of PowerShell is quite old (5.1), so don't
# try to use any new features.
#
# This script is specifically for windows, but has a similar structure to the
# unix `install.sh` script. If you modify this script, please check if the modifications
# should also apply to the unix one.
#
# General PowerShell notes:
#
# Since we have $ErrorActionPreference = "Stop" the script will stop at any `Write-Error`.
# Yeah, writing an error log counts as an error. Who could expect that? Regardless of how
# unintuitive it is we use `Write-Error` instead of `throw` because the error that is generated
# this way is more readable, because it refers to the call site of the function where the error
# was written and not to the `throw` statement itself in the code snippet that is output.
#
# Example error output with `throw` shows the `throw` statement itself, not the call site:
# ```
# Line |
#   74 |          throw "Command $cmd failed with exit code $LASTEXITCODE"
#      |          ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#      | Command curl failed with exit code 22
# ```
# Example error output with `Write-Error` shows the call site of the function which is more useful:
# ```
# Line |
#  174 |      Step curl$exe `
#      |      ~~~~~~~~~~~~~~~
#      | Command curl failed with exit code 22
# ```


$ErrorActionPreference = "Stop"

# This script isn't meant to be run from `master`, but if it is, then
# it will install the latest version be it a stable version or a pre-release.
# region replace-version unstable
$version = "0.2.1"
# endregion replace-version unstable

$toolchain = "nightly-2023-08-24"

# Log the command, execute, and fail if its exit code is non-zero.
# Surprisingly PowerShell can't do the exit code checks for us out of the box.
function Step {
    $cmd = $args[0]
    $rest = $args[1..$args.Length]

    # ASCII escape symbol
    $e = [char]0x1b

    # This is the unicode code of the symbol ❱.
    # Yeah, this is how you do unicode in PowerShell, yay -_-
    $run_symbol = [char]0x2771

    Write-Host "$e[32;1m$run_symbol $e[1;33m$cmd$e[0m $($rest -join ' ')"

    & $cmd @rest

    # Turns out `ErrorActionPreference` doesn't affect the behavior of external
    # processes. So if any process returns a non-zero exit code PowerShell will
    # still ignore this. Yeah.. Life is cruel, but at least PowerShell has
    # an experimental feature to fix this: `PSNativeCommandErrorActionPreference`.
    # Anyway, since we have to support pre-historic PowerShell, we'll do our own
    # error handling.
    if ($? -eq $false) {
        Write-Error "Command $cmd failed with exit code $LASTEXITCODE"
    }
}

function Unzip {
    param (
        [string]$bin,
        [string]$dest
    )
    $file_stem = "${bin}-${host_triple}"

    # We have to enter and exit from the temp dir because the destination path may be
    # relative, and we don't want that path to be relative to the temp dir.
    Step Push-Location $temp_dir
    Step Check-Sha256Sum $file_stem "zip"
    Step Pop-Location

    # There is `tar` on Windows installed by default, but it looks like different
    # environments have extremely different variations of tar installed.
    #
    # For example, my home laptop with Windows 11 has `bsdtar 3.5.2` installed.
    # It works fine with Windows paths out of the box. However, Windows Github Actions
    # runner has `tar (GNU tar) 1.34`, which somehow doesn't work with Windows paths.
    # It just doesn't eat a path with a drive letter and says "No such file or directory".
    #
    # Anyway, there is so much inconsistency between home Windows and Github Actions, that
    # it's just easier to use the PowerShell builtin utility that has always been there.
    Step Expand-Archive `
        -Force `
        -LiteralPath (Join-Path $temp_dir "$file_stem.zip") `
        -DestinationPath $dest
}

# There isn't mktemp on Windows, so we have to reinvent the wheel.
function New-TemporaryDirectory {
    $parent = [System.IO.Path]::GetTempPath()
    [string] $name = [System.Guid]::NewGuid()
    Step New-Item -ItemType Directory -Path (Join-Path $parent $name)
}

# There isn't sha256sum on Windows, so we have to reinvent the wheel.
function Check-Sha256Sum {
    param (
        [string]$file_stem,
        [string]$extension
    )

    $file = "$file_stem.$extension"

    $actual = Get-FileHash -Algorithm SHA256 -Path $file
    $expected = Get-Content "$file_stem.sha256"

    foreach ($line in $expected) {
        # Silence the output from the match, otherwise it prints `True` or `False`
        $null = $line -match '(\S+)\s*\*?(.*)'
        $expected_hash = $Matches[1]
        $expected_file = $Matches[2]

        if ($expected_file -ne $file) {
            continue
        }

        if ($actual.Hash -eq $expected_hash) {
            Write-Output "${file}: OK"
            return
        }


        Write-Error "Checksum verification failed for $file. Expected: $expected_hash, actual: $actual"
    }

    Write-Error "No checksum found for $file"
}

Write-Output "PowerShell version: $($PSVersionTable.PSVersion)"

# This script can run on unix too if you have PowerShell installed there.
# The only difference is that on Windows's old PowerShell 5 there is an alias
# `curl` for `Invoke-WebRequest`, and if you want to use the real curl, then
# you have to use `curl.exe`. That's a really perplexing design decision. Why
# would you alias `Invoke-WebRequest` to `curl` when it's not a real drop-in
# replacement for curl? Anyway.. PowerShell 7 removed that alias.
# More details here: https://stackoverflow.com/a/75867014
#
# Btw. this condition is written specically so that it can run on PowerShell 5,
# In PowerShell 7 there are automatic variables `$IsWindows`, `$IsMacOS`, `$IsLinux`.
$exe = if ([System.Environment]::OSVersion.Platform -eq "Win32NT") {
    ".exe"
} else {
    ""
}

Step curl$exe --version

Step rustup install --profile minimal --no-self-update $toolchain

$host_triple = (
    rustc +$toolchain --version --verbose `
    | Select-String -Pattern "host: (.*)" `
    | ForEach-Object { $_.Matches.Groups[1].Value }
)

$current_dir = (Get-Location).Path
$temp_dir = New-TemporaryDirectory

try {
    $files = "{cargo-marker,marker_rustc_driver}-$host_triple.{zip,sha256}"

    # Curl is available by default on windows, yay!
    # https://curl.se/windows/microsoft.html
    #
    # Download all files using a single TCP connection with HTTP2 multiplexing
    Step curl$exe `
        --location `
        --silent `
        --fail `
        --show-error `
        --retry 5 `
        --retry-connrefused `
        --remote-name `
        --output-dir $temp_dir `
        "https://github.com/rust-marker/marker/releases/download/v$version/$files"

    # There is a null coalescing operator in PowerShell 7, but that version
    # is too cutting edge for now.
    $cargo_home = if ($env:CARGO_HOME) {
        $env:CARGO_HOME
    } else {
        Join-Path $HOME ".cargo"
    }

    Unzip "cargo-marker" (Join-Path $cargo_home "bin")

    $sysroot = (rustc +$toolchain --print sysroot)
    Unzip "marker_rustc_driver" (Join-Path $sysroot "bin")

    # We use `+$toolchain` to make sure we don't try to install the default toolchain
    # in the workspace via the rustup proxy, but use the toolchain we just installed.
    Step cargo +$toolchain marker --version
} finally {
    # Go back to the original directory before removing the temp directory
    # otherwise it will fail because the temporary directory is in use.
    cd $current_dir

    Step Remove-Item -Force -Recurse $temp_dir
}

# You, my friend will be surprised. But the workability of this entire
# script depends on the presence of this comment. I'm not joking. If you remove
# this comment the script won't work. The try block higher won't be executed.
# That's the stupidest thing I've ever seen in my life, really.
#
# The reason is that if you pipe this script to a PowerShell interpreter, then
# that interpreter will not execute a multiline expression (which try block is)
# unless that expression is followed by two blank lines.
#
# You can test this with `Get-Content .\scripts\release\install.ps1 | powershell -command -`
#
# That's crazy, but this bug has been not fixed since 2017 when it was reported:
# https://github.com/PowerShell/PowerShell/issues/3223
