use crate::error::prelude::*;

use camino::Utf8PathBuf;
use std::path::PathBuf;

/// Trait supports conversion `Vec<u8> -> String` and `PathBuf -> Utf8PathBuf`
pub trait IntoUtf8 {
    type Output;

    fn into_utf8(self) -> Result<Self::Output>;
}

impl IntoUtf8 for Vec<u8> {
    type Output = String;

    fn into_utf8(self) -> Result<Self::Output> {
        String::from_utf8(self).map_err(|err| {
            Error::root(format!(
                "Failed to convert to UTF-8 encoded string (dumped it on the line bellow):\n\
                ---\n{}\n---",
                String::from_utf8_lossy(err.as_bytes())
            ))
        })
    }
}

impl IntoUtf8 for PathBuf {
    type Output = Utf8PathBuf;

    fn into_utf8(self) -> Result<Self::Output> {
        Utf8PathBuf::try_from(self).map_err(|err| {
            Error::root(format!(
                "Failed to convert to UTF-8 encoded path (dumped it on the line bellow):\n\
                ---\n{}\n---",
                err.as_path().display()
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Display;

    use super::*;
    use expect_test::{expect, Expect};

    fn assert_into_utf8<T>(actual: T, expect: &Expect)
    where
        T: IntoUtf8,
        T::Output: Display,
    {
        let actual = actual
            .into_utf8()
            .map(|ok| ok.to_string())
            .unwrap_or_else(|err| format!("Error: {err}"));

        expect.assert_eq(&actual);
    }

    #[test]
    fn test_into_utf8_success() {
        assert_into_utf8(vec![97, 98, 99u8], &expect!["abc"]);
    }

    #[test]
    fn test_into_utf8_fail() {
        assert_into_utf8(
            vec![97, 98, 255u8],
            &expect![[r"
                Error: Failed to convert to UTF-8 encoded string (dumped it on the line bellow):
                ---
                ab�
                ---"]],
        );
    }

    #[test]
    fn test_pathbuf_into_utf8_success() {
        assert_into_utf8(PathBuf::from("/My/Custom/Path"), &expect!["/My/Custom/Path"]);
    }
}
