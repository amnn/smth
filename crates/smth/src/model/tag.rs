// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Single-letter session tags and their compact membership set.

use std::fmt;
use std::ops::BitOr;
use std::str::FromStr;

use anyhow::ensure;

use crate::config::Alphabet;

/// A validated lowercase ASCII session tag.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Tag(u8);

/// Membership in the 26 session tags, stored in the low bits of a word.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Tags(u32);

/// A session navigation ring selected by a letter or the attention sigil.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Target {
    Tag(Tag),
    Attention,
}

impl Tag {
    /// Return the lowercase letter used in commands and serialized output.
    pub fn letter(self) -> char {
        char::from(b'a' + self.0)
    }
}

impl Tags {
    /// Return whether this set contains a tag.
    pub fn contains(self, tag: Tag) -> bool {
        self.0 & (1 << tag.0) != 0
    }

    /// Render memberships in tag order using the display alphabet, without changing identity.
    pub fn display(self, alphabet: &Alphabet) -> String {
        self.iter().map(|tag| alphabet[tag.0 as usize]).collect()
    }

    /// Iterate over members in alphabetical order.
    pub fn iter(self) -> impl Iterator<Item = Tag> {
        (0..26).map(Tag).filter(move |tag| self.contains(*tag))
    }

    /// Set one membership without affecting any other tag.
    pub fn set(&mut self, tag: Tag, enabled: bool) {
        if enabled {
            self.0 |= 1 << tag.0;
        } else {
            self.0 &= !(1 << tag.0);
        }
    }

    /// Toggle one membership.
    pub fn toggle(&mut self, tag: Tag) {
        self.0 ^= 1 << tag.0;
    }
}

impl Target {
    /// Whether a session belongs to this navigation target.
    pub fn matches(self, session: &super::Session) -> bool {
        match self {
            Self::Tag(tag) => session.tags().is_some_and(|tags| tags.contains(tag)),
            Self::Attention => session.flag() == Some(true) || session.has_alerts(),
        }
    }
}

impl BitOr for Tags {
    type Output = Self;

    /// Return the union of both membership sets.
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.letter().fmt(f)
    }
}

impl fmt::Display for Tags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for tag in self.iter() {
            tag.fmt(f)?;
        }

        Ok(())
    }
}

impl FromStr for Tag {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> anyhow::Result<Self> {
        ensure!(
            value.len() == 1 && value.as_bytes()[0].is_ascii_lowercase(),
            "tag must be one lowercase letter a-z"
        );

        Ok(Self(value.as_bytes()[0] - b'a'))
    }
}

impl FromStr for Target {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> anyhow::Result<Self> {
        if value == "-" {
            Ok(Self::Attention)
        } else {
            value.parse().map(Self::Tag)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_preserves_membership_order() {
        let fraktur = Alphabet::try_from("𝖆𝖇𝖈𝖉𝖊𝖋𝖌𝖍𝖎𝖏𝖐𝖑𝖒𝖓𝖔𝖕𝖖𝖗𝖘𝖙𝖚𝖛𝖜𝖝𝖞𝖟".to_owned()).unwrap();
        let mut tags = Tags::default();
        tags.set("z".parse().unwrap(), true);
        tags.set("a".parse().unwrap(), true);
        assert_eq!(tags.display(&fraktur), "𝖆𝖟");
        assert_eq!(tags.display(&Alphabet::default()), "az");
    }

    #[test]
    fn membership_is_independent_and_ordered() {
        let a = "a".parse().unwrap();
        let z = "z".parse().unwrap();

        let mut tags = Tags::default();
        tags.set(z, true);
        tags.set(a, true);
        tags.set(a, true);

        let s: String = tags.iter().map(|t| t.letter()).collect();
        assert_eq!(s, "az");

        tags.set(a, false);
        tags.set(a, false);
        assert!(!tags.contains(a));
        assert!(tags.contains(z));

        tags.toggle(z);
        assert_eq!(tags, Tags::default());
    }

    #[test]
    fn parsing_requires_one_lowercase_ascii_letter() {
        for value in ["", "A", "aa", "é", "1", " a", "a\n"] {
            assert!(value.parse::<Tag>().is_err(), "{value:?}");
        }

        for letter in 'a'..='z' {
            let tag: Tag = letter.to_string().parse().unwrap();
            assert_eq!(tag.letter(), letter);
        }
    }

    #[test]
    fn union_combines_memberships_without_duplicates() {
        let mut left = Tags::default();
        let mut right = Tags::default();
        for tag in ["a", "m"] {
            left.set(tag.parse().unwrap(), true);
        }
        for tag in ["m", "z"] {
            right.set(tag.parse().unwrap(), true);
        }

        assert_eq!((left | right).to_string(), "amz");
        assert_eq!(left | right, right | left);
        assert_eq!(left | left, left);
        assert_eq!(left | Tags::default(), left);
        assert_eq!(Tags::default() | right, right);
    }
}
