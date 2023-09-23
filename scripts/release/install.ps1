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

$ErrorActionPreference = "Stop"

# This script isn't meant to be run from `master`, but if it is, then
# it will install the latest version be it a stable version or a pre-release.
# region replace-version unstable
$version = "0.2.1"
# endregion replace-version unstable

$toolchain = "nightly-2023-08-24"

function With-Log {
    Write-Output "> $($args -join ' ')"

    $cmd = $args[0]
    $rest = $args[1..$args.Length]

    & $cmd @rest
}

function Extract-TarGz {
    param (
        [string]$bin,
        [string]$dest
    )
    $file_stem = "${bin}-${host_triple}"

    With-Log cd $temp_dir

    Check-Sha256Sum $file_stem "tar.gz"

    With-Log tar --extract --file (Join-Path $temp_dir "$file_stem.tar.gz") --directory $dest
}

# There isn't mktemp on Windows, so we have to reinvent the wheel.
function New-TemporaryDirectory {
    $parent = [System.IO.Path]::GetTempPath()
    [string] $name = [System.Guid]::NewGuid()
    New-Item -ItemType Directory -Path (Join-Path $parent $name)
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
        $line -match '(\S+)\s*\*?(.*)'
        $expected_hash = $Matches[1]
        $expected_file = $Matches[2]

        if ($expected_file -ne $file) {
            continue
        }

        if ($actual.Hash -eq $expected_hash) {
            Write-Output "${file}: OK"
            return
        }

        throw "Checksum verification failed for $file. Expected: $expected_hash, actual: $actual"
    }

    throw "No checksum found for $file"
}

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

With-Log rustup install --profile minimal --no-self-update $toolchain

$host_triple = (
    rustc +$toolchain --version --verbose `
    | Select-String -Pattern "host: (.*)" `
    | ForEach-Object { $_.Matches.Groups[1].Value }
)

$current_dir = (Get-Location).Path
$temp_dir = New-TemporaryDirectory

try {
    $files = "{cargo-marker,marker_rustc_driver}-$host_triple.{tar.gz,sha256}"

    # Curl is available by default on windows, yay!
    # https://curl.se/windows/microsoft.html
    #
    # Download all files using a single TCP connection with HTTP2 multiplexing
    With-Log curl$exe `
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

    Extract-TarGz "cargo-marker" (Join-Path $cargo_home "bin")

    $sysroot = (rustc +$toolchain --print sysroot)
    Extract-TarGz "marker_rustc_driver" (Join-Path $sysroot "bin")
} finally {
    Write-Output "Removing the temp directory $temp_dir"

    # Go back to the original directory before removing the temp directory
    # otherwise it will fail because the temporary directory is in use.
    cd $current_dir

    Remove-Item -Force -Recurse $temp_dir
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
