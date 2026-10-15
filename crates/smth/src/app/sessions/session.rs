// Copyright (c) Ashok Menon
// SPDX-License-Identifier: Apache-2.0

//! Rendering for individual sessions.

use ratatui::style::Style;
use ratatui::style::Stylize as _;
use ratatui::text::Line;
use ratatui::text::Span;
use unicode_width::UnicodeWidthChar as _;
use unicode_width::UnicodeWidthStr as _;

use crate::app::agent;
use crate::app::component::row::Row;
use crate::app::highlight::Highlight;
use crate::app::span::push_repo_path_spans;
use crate::app::span::tag_span;
use crate::model::session::Base;
use crate::model::session::DELIM_SUFFIX;
use crate::model::session::NAME_WIDTH;
use crate::model::session::Session;

const HIGHLIGHT: Style = Style::new().blue().bold();

const SIGIL_DELETE: &str = "×";

/// Render `session` as a session-list row.
pub(super) fn row(
    sigil: char,
    session: &Session,
    highlighted: bool,
    deleting: bool,
    matches: &[u32],
) -> Row {
    let mut hl = Highlight::new(matches.to_vec(), |_| HIGHLIGHT);
    let mut line = Line::default();
    push_session_name_spans(&mut line, session, &mut hl, deleting, highlighted);

    let mut column = line.width();
    let space = NAME_WIDTH.saturating_sub(column);
    let tags: String = session
        .tags()
        .map_or_default(|ts| contracted(&ts.to_string(), space));

    if !tags.is_empty() {
        line += Span::raw(" ".repeat(NAME_WIDTH - column - tags.width()));
        line += tag_span(tags, highlighted);
        column = NAME_WIDTH;
    }

    if let Some(repo) = session.repo() {
        line += Span::raw(" ".repeat(NAME_WIDTH.saturating_sub(column) + 1));
        // Matcher offsets count characters, independently of the decorative tags' cell widths.
        let repo_offset = session.name().chars().count().max(NAME_WIDTH) + 1;
        hl.skip_to(repo_offset as u32);
        push_repo_path_spans(&mut line, &repo, &mut hl);
    }

    let row = Row::new(line);
    let row = if let Some(base) = session.new_base() {
        let label = match base {
            Base::NewRepo(_) => " repo ",
            Base::Repo(_) => " workspace ",
            Base::Cwd(_) => " tmux ",
        };

        row.with_overlay(Line::from(Span::raw(label).dim()).right_aligned())
    } else if let Some(agents) = session.agents() {
        row.with_overlay(agent::summary(agents, highlighted))
    } else {
        row
    };

    if deleting {
        let sigil = Span::raw(SIGIL_DELETE);
        return row.with_sigil(if highlighted {
            sigil.on_light_red()
        } else {
            sigil.light_red()
        });
    }

    let Some(flagged) = session.flag() else {
        return row;
    };

    let alerts = session.has_alerts();
    let sigil = Span::raw(sigil.to_string());
    if alerts && highlighted {
        row.with_sigil(sigil.on_light_yellow())
    } else if alerts {
        row.with_sigil(sigil.light_yellow())
    } else if flagged && highlighted {
        row.with_sigil(sigil.on_light_blue())
    } else if flagged {
        row.with_sigil(sigil.light_blue())
    } else {
        row.with_sigil(sigil.dim())
    }
}

/// Fit tags into available cells, ending with an ellipsis or an alignment space.
fn contracted(text: &str, mut available: usize) -> String {
    if text.is_empty() || available < 2 {
        return String::new();
    }

    // Pre-reserve the space for the padding to the front and either the ellipsis or the alignment
    // space at the end.
    available -= 2;

    if text.width() < available {
        return format!(" {text} ");
    }

    let mut result = String::new();
    for letter in text.chars() {
        let width = letter.width().unwrap_or_default();
        if width > available {
            break;
        }

        result.push(letter);
        available -= width;
    }

    result.push('…');
    result
}

/// Push styled session name spans, dimming a disambiguation suffix when present.
fn push_session_name_spans<'a, F: Fn(Style) -> Style>(
    spans: &mut impl Extend<Span<'a>>,
    session: &Session,
    hl: &mut Highlight<F>,
    deleting: bool,
    highlighted: bool,
) {
    let name = session.name();

    let name_style = if deleting && highlighted {
        Style::new().on_light_red().bold()
    } else if deleting {
        Style::new().light_red().bold()
    } else {
        Style::new()
    };

    let suffix_style = name_style.dim();

    let Some((prefix, suffix)) = name.rsplit_once(DELIM_SUFFIX) else {
        spans.extend(hl.highlight(Span::styled(name, name_style)));
        return;
    };

    spans.extend(hl.highlight(Span::styled(prefix.to_owned(), name_style)));
    spans.extend(hl.highlight(Span::styled(DELIM_SUFFIX, suffix_style)));
    spans.extend(hl.highlight(Span::styled(suffix.to_owned(), suffix_style)));
}
