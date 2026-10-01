// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context as _;

/// Expand references once, preserving argument boundaries and shell-owned escaped dollars.
/// Missing variables become empty strings; a dollar without a name remains literal.
///
/// Returns an error if a braced variable reference has no closing brace.
pub(crate) fn expand(
    mut input: &str,
    mut substitute: impl FnMut(&str) -> Option<String>,
) -> anyhow::Result<String> {
    let mut output = String::with_capacity(input.len());
    while let Some(ix) = input.find('$') {
        output.push_str(&input[..ix]);
        input = &input[ix + 1..];
        if let Some(rest) = input.strip_prefix('$') {
            output.push('$');
            input = rest;
            continue;
        }

        let name = if let Some(rest) = input.strip_prefix('{') {
            let end = rest.find('}').context("unterminated variable reference")?;
            input = &rest[end + 1..];
            &rest[..end]
        } else {
            let end = input
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(input.len());
            if end == 0 {
                output.push('$');
                continue;
            }
            let (name, rest) = input.split_at(end);
            input = rest;
            name
        };

        output.push_str(&substitute(name).unwrap_or_default());
    }
    output.push_str(input);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_once_without_splitting() {
        let expanded = expand("$A/${B}/$MISSING/$$A/$1/end$", |name| match name {
            "A" => Some("two words".into()),
            "B" => Some("$A".into()),
            "1" => Some("one".into()),
            _ => None,
        })
        .unwrap();
        assert_eq!(expanded, "two words/$A//$A/one/end$");
    }

    #[test]
    fn rejects_unclosed_braces() {
        assert!(expand("${OPEN", |_| None).is_err());
    }
}
