// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Ordered, independently directed session comparators.

use std::cmp::Ordering;
use std::cmp::Reverse;
use std::fmt;
use std::str::FromStr;

use anyhow::bail;
use anyhow::ensure;
use nucleo::Config;
use nucleo::Item;
use nucleo::Matcher;
use nucleo::Snapshot;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;

use crate::model::session::Session;

/// Session ordering from highest to lowest priority.
///
/// Every key occurs once, including disabled keys. Disabled keys retain their positions but do not
/// participate in comparison. Equal sessions retain their input order when sorted stably.
#[derive(Clone, Debug, DeserializeFromStr, Eq, PartialEq, SerializeDisplay)]
pub struct Order {
    entries: Vec<Criterion>,
}

/// One comparator and its independently enabled direction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Criterion {
    pub(crate) key: Key,
    pub(crate) direction: Option<Direction>,
}

/// Direction of increasing values on the page.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Direction {
    Ascending,
    Descending,
}

/// Available comparators, also defining the append order for omitted keys.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Key {
    Live,
    Recency,
    Name,
    Attention,
}

impl Order {
    /// Borrow all criteria in priority order, including disabled criteria.
    pub(crate) fn entries(&self) -> &[Criterion] {
        &self.entries
    }

    /// Rank queries by fuzzy score, breaking score ties with the configured stack.
    /// Without a query, apply only the stack and preserve discovery order for complete ties.
    pub(crate) fn matches<'s>(&self, snapshot: &'s Snapshot<Session>) -> Vec<Item<'s, Session>> {
        let mut items: Vec<_> = snapshot.matched_items(..).collect();
        items.sort_by(|left, right| self.compare(left.data, right.data));

        let pattern = snapshot.pattern();
        if !pattern.is_empty() {
            let mut matcher = Matcher::new(Config::DEFAULT);
            items.sort_by_cached_key(|item| {
                Reverse(
                    pattern
                        .score(item.matcher_columns, &mut matcher)
                        .expect("snapshot items match its pattern"),
                )
            });
        }

        items
    }

    /// Compare sessions using only enabled criteria, falling through on equality.
    pub(crate) fn compare(&self, left: &Session, right: &Session) -> Ordering {
        for criterion in &self.entries {
            let Some(direction) = criterion.direction else {
                continue;
            };

            let ordering = match criterion.key {
                Key::Live => left.is_live().cmp(&right.is_live()),
                Key::Name => left.name().cmp(&right.name()),
                Key::Attention => left.has_alerts().cmp(&right.has_alerts()),
                Key::Recency => left.recency().cmp(&right.recency()),
            };

            let ordering = match direction {
                Direction::Ascending => ordering,
                Direction::Descending => ordering.reverse(),
            };

            if ordering != Ordering::Equal {
                return ordering;
            }
        }

        Ordering::Equal
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut prefix = "";
        for entry in &self.entries {
            let Some(direction) = entry.direction else {
                continue;
            };

            let sign = match direction {
                Direction::Ascending => '+',
                Direction::Descending => '-',
            };

            let name = match entry.key {
                Key::Live => "live",
                Key::Recency => "recency",
                Key::Name => "name",
                Key::Attention => "attn",
            };

            write!(f, "{prefix}{sign}{name}")?;
            prefix = " ";
        }

        Ok(())
    }
}

impl FromStr for Order {
    type Err = anyhow::Error;

    /// Parse signed comparator names; omitted keys are disabled and appended deterministically.
    fn from_str(value: &str) -> anyhow::Result<Self> {
        let mut entries: Vec<Criterion> = Vec::new();
        for token in value.split_whitespace() {
            let (direction, name) = if let Some(name) = token.strip_prefix('+') {
                (Direction::Ascending, name)
            } else if let Some(name) = token.strip_prefix('-') {
                (Direction::Descending, name)
            } else {
                bail!("comparator '{token}' must start with + or -");
            };

            let key = match name {
                "live" => Key::Live,
                "recency" => Key::Recency,
                "name" => Key::Name,
                "attn" => Key::Attention,
                _ => bail!("unknown comparator '{name}'; expected live, recency, name, or attn"),
            };

            ensure!(
                !entries.iter().any(|e| e.key == key),
                "duplicate ordering comparator '{name}'"
            );

            entries.push(Criterion {
                key,
                direction: Some(direction),
            });
        }

        // Add all the keys that were not explicitly mentioned at the end, as disabled criteria.
        for key in [Key::Live, Key::Recency, Key::Name, Key::Attention] {
            if !entries.iter().any(|entry| entry.key == key) {
                entries.push(Criterion {
                    key,
                    direction: None,
                });
            }
        }

        Ok(Self { entries })
    }
}

impl Default for Order {
    fn default() -> Self {
        Self {
            entries: vec![
                Criterion {
                    key: Key::Live,
                    direction: Some(Direction::Descending),
                },
                Criterion {
                    key: Key::Recency,
                    direction: Some(Direction::Descending),
                },
                Criterion {
                    key: Key::Name,
                    direction: Some(Direction::Ascending),
                },
                Criterion {
                    key: Key::Attention,
                    direction: None,
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    use super::*;
    use crate::cmd::tmux;
    use crate::model::session::LiveKind;
    use crate::model::session::RepoKind;

    /// Assert sorted names at the invocation's source location.
    macro_rules! assert_names {
        ($encoding:expr, $sessions:expr, $expected:expr $(,)?) => {{
            let encoding = $encoding;
            let order: Order = encoding.parse().unwrap();
            let mut sessions: Vec<Session> = ($sessions).to_vec();
            sessions.sort_by(|left, right| order.compare(left, right));
            let names: Vec<_> = sessions.iter().map(Session::name).collect();
            assert_eq!(names, $expected, "{encoding}");
        }};
    }

    /// Construct a live fixture with explicit tmux attachment metadata.
    fn live(name: &str, time: Option<u64>, attached: bool) -> Session {
        LiveKind::new(
            name.to_owned(),
            None,
            tmux::SessionInfo {
                agents: BTreeMap::default(),
                alerts: BTreeSet::default(),
                attached,
                flagged: false,
                last_attached: time,
                repo: None,
            },
        )
        .into()
    }

    #[test]
    fn parses_stack_and_appends_omitted_keys() {
        assert_eq!(
            "-live -recency +name".parse::<Order>().unwrap(),
            Order::default()
        );

        let order: Order = " \t-name\n+live ".parse().unwrap();
        assert_eq!(
            order.entries,
            vec![
                Criterion {
                    key: Key::Name,
                    direction: Some(Direction::Descending)
                },
                Criterion {
                    key: Key::Live,
                    direction: Some(Direction::Ascending)
                },
                Criterion {
                    key: Key::Recency,
                    direction: None
                },
                Criterion {
                    key: Key::Attention,
                    direction: None
                },
            ]
        );

        for invalid in [
            "live",
            "!live",
            "+last",
            "-last",
            "*name",
            "+",
            "--live",
            "+unknown",
            "+name -name",
            "-live -live",
            "＋name",
        ] {
            assert!(invalid.parse::<Order>().is_err(), "accepted {invalid:?}");
        }
    }

    #[test]
    fn attached_is_maximal_even_without_history() {
        let sessions = [
            live("attached-old", Some(1), true),
            live("missing", None, false),
            live("new", Some(u64::MAX), false),
            live("attached-missing", None, true),
            live("old", Some(1), false),
        ];

        assert_names!(
            "-recency",
            &sessions,
            &["attached-old", "attached-missing", "new", "old", "missing"],
        );

        assert_names!(
            "+recency",
            &sessions,
            &["missing", "old", "new", "attached-old", "attached-missing"],
        );

        assert_names!(
            "-recency +name",
            &sessions,
            &["attached-missing", "attached-old", "new", "old", "missing"],
        );
    }

    #[test]
    fn default_order_places_unvisited_live_sessions_before_repos() {
        let repo: Session = RepoKind::new(None, "alpha".into(), "alpha".into()).into();
        let sessions = [
            repo,
            live("never", None, false),
            live("visited", Some(1), false),
            live("attached", None, true),
        ];
        assert_names!(
            "-live -recency +name",
            &sessions,
            &["attached", "visited", "never", "alpha"],
        );
        assert_names!(
            "-live +recency +name",
            &sessions,
            &["never", "visited", "attached", "alpha"],
        );
    }

    #[test]
    fn ties_have_no_hidden_name_or_attachment_comparator() {
        let sessions = [live("z", Some(5), false), live("a", Some(5), false)];
        assert_names!("-recency", &sessions, &["z", "a"]);
        assert_names!("+recency", &sessions, &["z", "a"]);
        assert_names!("-recency +name", &sessions, &["a", "z"]);
        let missing = [live("z", None, false), live("a", None, false)];
        assert_names!("+recency +name", &missing, &["a", "z"]);
        assert_names!("-recency +name", &missing, &["a", "z"]);
        assert_names!("", &sessions, &["z", "a"]);
        assert_names!(" \n\t", &sessions, &["z", "a"]);
    }

    #[test]
    fn priority_and_boolean_directions_are_independent() {
        let repo: Session = RepoKind::new(None, "a".into(), "a".into()).into();
        let sessions = [repo, live("z", None, false)];
        assert_names!("-live", &sessions, &["z", "a"]);
        assert_names!("+live", &sessions, &["a", "z"]);
        assert_names!("+name -live", &sessions, &["a", "z"]);
        assert_names!("-live +name", &sessions, &["z", "a"]);
        assert_names!("", &sessions, &["a", "z"]);
    }

    #[test]
    fn attention_compares_presence_not_count_or_manual_flag() {
        let session = |name: &str, alerts: &[&str], flagged| -> Session {
            LiveKind::new(
                name.to_owned(),
                None,
                tmux::SessionInfo {
                    agents: BTreeMap::default(),
                    alerts: alerts.iter().map(|s| (*s).to_owned()).collect(),
                    attached: false,
                    flagged,
                    last_attached: None,
                    repo: None,
                },
            )
            .into()
        };

        let sessions = [
            session("flagged", &[], true),
            session("two", &["@1", "@2"], false),
            session("one", &["@3"], false),
        ];

        assert_names!("-attn", &sessions, &["two", "one", "flagged"]);
        assert_names!("+attn", &sessions, &["flagged", "two", "one"]);
        assert_names!("-attn +name", &sessions, &["one", "two", "flagged"]);
    }
}
