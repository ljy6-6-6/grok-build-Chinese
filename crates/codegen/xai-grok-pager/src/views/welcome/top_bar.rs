//! Top bar component: renders cwd and git info.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use std::path::{Path, PathBuf};

use crate::git_info;
use crate::render::line_utils::truncate_line;
use crate::theme::Theme;
use crate::views::location::{location_parts_with_locale, worktree_badge_with_locale};

pub fn render_top_bar(
    area: Rect,
    buf: &mut Buffer,
    theme: &Theme,
    announcement: Option<&xai_grok_announcements::RemoteAnnouncement>,
    locale: Option<&crate::locale::LocaleContext>,
) {
    let line = truncate_line(
        location_line_with_locale(theme, locale),
        area.width as usize,
    );
    let line_width = line.width() as u16;
    buf.set_line(area.x, area.y, &line, line_width.min(area.width));

    if let Some(a) = announcement
        && let Some(text) = a.message.as_deref()
        && area.height > 1
    {
        let text_style = Style::default().fg(theme.text_primary);
        let line = Line::from(Span::styled(text, text_style));
        Paragraph::new(line).render(
            Rect {
                y: area.y + 1,
                height: area.height.saturating_sub(1),
                ..area
            },
            buf,
        );
    }
}

/// Build the `{git branch} {worktree} {cwd}` line for the welcome top bar, reading the live process cwd.
/// The caller width-truncates the returned line.
#[allow(dead_code)]
pub(crate) fn location_line(theme: &Theme) -> Line<'static> {
    location_line_with_locale(theme, None)
}

pub(crate) fn location_line_with_locale(
    theme: &Theme,
    locale: Option<&crate::locale::LocaleContext>,
) -> Line<'static> {
    location_line_at_with_locale(theme, &process_cwd(), locale)
}

pub(crate) fn location_line_at(theme: &Theme, cwd: &Path) -> Line<'static> {
    location_line_at_with_locale(theme, cwd, None)
}

pub(crate) fn location_line_at_with_locale(
    theme: &Theme,
    cwd: &Path,
    locale: Option<&crate::locale::LocaleContext>,
) -> Line<'static> {
    let info_style = Style::default().fg(theme.gray);
    let parts = location_parts_with_locale(cwd, locale);

    let mut spans: Vec<Span> = Vec::new();
    if let Some(branch) = parts.branch.as_deref() {
        let icon = git_info::branch_icon();
        let git_style = Style::default()
            .fg(theme.text_primary)
            .add_modifier(Modifier::DIM);
        spans.push(Span::styled(format!("{icon} {branch}"), git_style));
        spans.push(Span::styled(" ", info_style));
    }
    if parts.is_worktree {
        spans.push(worktree_badge_with_locale(theme, locale));
    }
    let cwd_style = Style::default().fg(theme.gray_dim);
    spans.push(Span::styled(parts.cwd_display, cwd_style));
    Line::from(spans)
}

fn process_cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
