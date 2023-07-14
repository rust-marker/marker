use std::{env, process::exit};

const BUILD_ATTEMPT_MESSAGE: &str = r#"
error: unexpected attempt to build Marker's rustc driver

This driver requires a specific toolchain version and should be stored with the
corresponding toolchain. Try installing the driver with `cargo_marker` instead:

```
# Install `cargo-marker`
cargo install cargo_marker

# Install the toolchain and driver
cargo marker setup --auto-install-toolchain
```

If you know what you're doing, you can also disable this check by setting the
`MARKER_ALLOW_DRIVER_BUILD` environment value.
"#;

fn main() {
    let profile = env::var("PROFILE").expect("the `PROFILE` environment value is not set");
    if profile == "release" && env::var_os("MARKER_ALLOW_DRIVER_BUILD").is_none() {
        eprintln!("{BUILD_ATTEMPT_MESSAGE}");
        exit(1);
    }

    rustc_tools_util::setup_version_info!();

    // Don't rebuild even if nothing changed
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=MARKER_ALLOW_DRIVER_BUILD");
    println!("cargo:rerun-if-env-changed=PROFILE");
}
