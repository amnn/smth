// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Validated 26-character display alphabets.

use std::ops::Index;

use anyhow::anyhow;
use anyhow::ensure;
use serde::Deserialize;
use serde::Serialize;
use unicode_width::UnicodeWidthChar as _;

/// Number of characters in an alphabet.
const COUNT: usize = 26;

/// One visible, non-whitespace character for each letter a-z, in that order.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(try_from = "String", into = "String")]
pub struct Alphabet([char; COUNT]);

impl Default for Alphabet {
    fn default() -> Self {
        Self(std::array::from_fn(|index| char::from(b'a' + index as u8)))
    }
}

impl From<Alphabet> for String {
    fn from(alphabet: Alphabet) -> Self {
        alphabet.0.into_iter().collect()
    }
}

impl Index<usize> for Alphabet {
    type Output = char;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl TryFrom<String> for Alphabet {
    type Error = anyhow::Error;

    /// Parse a display alphabet in letter order.
    ///
    /// Accepts one character to start a consecutive Unicode code-point range, or 26 explicit
    /// characters. Returns an error for invalid code points, whitespace, or non-positive widths.
    fn try_from(value: String) -> anyhow::Result<Self> {
        let letters: Vec<_> = value.chars().collect();
        let letters = if let [first] = letters[..] {
            (0..COUNT)
                .flat_map(|offset| char::from_u32(first as u32 + offset as u32))
                .collect()
        } else {
            letters
        };

        let letters: [char; COUNT] = letters.try_into().map_err(|_| {
            anyhow!(
                "alphabet must contain exactly {COUNT} characters, or a single character that \
                represents the start of a consecutive range of {COUNT} valid characters"
            )
        })?;

        for l in &letters {
            ensure!(
                !l.is_whitespace() && l.width().is_some_and(|w| w > 0),
                "alphabet characters must not be whitespace and must have non-zero width"
            );
        }

        Ok(Self(letters))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_single_characters_to_consecutive_code_points() {
        for explicit in [
            "abcdefghijklmnopqrstuvwxyz",
            "𝖆𝖇𝖈𝖉𝖊𝖋𝖌𝖍𝖎𝖏𝖐𝖑𝖒𝖓𝖔𝖕𝖖𝖗𝖘𝖙𝖚𝖛𝖜𝖝𝖞𝖟",
            "ａｂｃｄｅｆｇｈｉｊｋｌｍｎｏｐｑｒｓｔｕｖｗｘｙｚ",
        ] {
            let first = explicit.chars().next().unwrap().to_string();
            let alphabet = Alphabet::try_from(first).unwrap();
            assert_eq!(String::from(alphabet), explicit);
        }
    }

    #[test]
    fn preserves_explicit_nonconsecutive_alphabets() {
        let explicit = "zyxwvutsrqponmlkjihgfedcba";
        let alphabet = Alphabet::try_from(explicit.to_owned()).unwrap();
        assert_eq!(String::from(alphabet), explicit);
    }

    #[test]
    fn rejects_invalid_alphabets() {
        for alphabet in [
            "",
            "abc",
            "abcdefghijklmnopqrstuvwxyz!",
            "abcdefghijklmnopqrstuvwxy ",
            "abcdefghijklmnopqrstuvwxy\n",
            "abcdefghijklmnopqrstuvwxy\u{301}",
            " ",
            "\u{301}",
            "\u{d7ff}",
            "\u{10ffff}",
        ] {
            assert!(
                Alphabet::try_from(alphabet.to_owned()).is_err(),
                "{alphabet:?}"
            );
        }
    }
}
