use std::path::Path;
use std::path::PathBuf;

pub async fn canonicalize(path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    let _path = path.as_ref().to_owned();
    Ok(PathBuf::from("./duck"))
}

fn main() {}
