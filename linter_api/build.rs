use std::process::Command;

fn main() {
    let output = Command::new("rustc")
        .arg("-Vv")
        .output()
        .expect("unable to load rustc version, please ensure that rustc is in the path");
    let version = String::from_utf8_lossy(&output.stdout).replace('\n', "|");
    println!("cargo:rustc-env=RUSTC_VERSION={}", version);
}
