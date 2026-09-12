use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::app::app_view::SessionPickerEntry;
use crate::theme::Theme;

/// Which surface a picker fetch was issued for.
/// Results route back to the requesting host's storage only; a live picker on another host never absorbs them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionPickerHost {
    /// Welcome-screen picker (`session_picker_*` fields on `AppView`).
    Welcome,
    /// `/resume` modal on the active agent (`ActiveModal::SessionPicker`).
    AgentModal,
    /// Dashboard picker (`AppView::dashboard_session_picker`).
    Dashboard,
}

/// Shared by dashboard picker paint and input so a copy change cannot update only one site.
pub(crate) const DASHBOARD_PICKER_TITLE: &str = "Open session";

/// Paint and input routing must resolve the same display title.
pub(crate) fn dashboard_picker_title(
    locale: Option<&crate::locale::LocaleContext>,
) -> &'static str {
    locale
        .map(|locale| {
            locale.named_static_text("session_picker.open_session", DASHBOARD_PICKER_TITLE)
        })
        .unwrap_or(DASHBOARD_PICKER_TITLE)
}

/// State for one session-picker incarnation.
/// Host-agnostic: everything a picker accumulates between open and dismiss, nothing about how a host renders it or maps its keys.
#[derive(Debug)]
pub struct SessionPickerSurface {
    /// Incarnation identity; results apply only when it matches.
    pub generation: u64,
    pub state: crate::views::picker::PickerState,
    pub window: crate::views::modal_window::ModalWindowState,
    pub entries: Option<Vec<crate::app::app_view::SessionPickerEntry>>,
    pub loading: bool,
    pub lanes: crate::views::session_picker::SessionPickerLanes,
    pub content_results: Option<Vec<xai_grok_shell::extensions::session_search::SearchSessionHit>>,
    pub content_loading: bool,
    /// Per-surface counters; the dashboard host does not share the welcome picker's `session_picker_list_seq` / `session_picker_deep_search_seq`.
    pub list_seq: u64,
    pub deep_search_seq: u64,
    /// Invalidates in-flight card-detail reads when this surface's rows or filters change.
    pub detail_seq: u64,
    pub entries_query: Option<String>,
    pub source_filter: crate::views::session_picker::SourceFilter,
    pub pending_delete: Option<crate::views::session_picker::PendingDelete>,
}

impl SessionPickerSurface {
    #[must_use]
    pub fn new(generation: u64) -> Self {
        Self {
            generation,
            state: crate::views::picker::PickerState::default(),
            window: crate::views::modal_window::ModalWindowState::new(),
            entries: None,
            loading: false,
            lanes: Default::default(),
            content_results: None,
            content_loading: false,
            list_seq: 0,
            deep_search_seq: 0,
            detail_seq: 0,
            entries_query: None,
            source_filter: Default::default(),
            pending_delete: None,
        }
    }
}

pub(crate) enum SessionPickerRenderMode<'a> {
    Fullscreen,
    Modal {
        window: &'a mut crate::views::modal_window::ModalWindowState,
        title: &'a str,
    },
}

pub(crate) struct SessionPickerRenderCtx<'a> {
    pub(crate) state: &'a mut crate::views::picker::PickerState,
    pub(crate) sessions: Option<&'a [SessionPickerEntry]>,
    pub(crate) cwd: &'a std::path::Path,
    pub(crate) loading: bool,
    pub(crate) pending_hint: Option<crate::views::shortcuts_bar::PendingHint>,
    pub(crate) shortcuts_area: Option<Rect>,
    pub(crate) content_results:
        Option<&'a [xai_grok_shell::extensions::session_search::SearchSessionHit]>,
    pub(crate) content_loading: bool,
    pub(crate) entries_query: Option<&'a str>,
    pub(crate) tick: u64,
    pub(crate) grouped: bool,
    pub(crate) source_filter: crate::views::session_picker::SourceFilter,
    pub(crate) pending_delete: bool,
    pub(crate) chat_mode: bool,
    pub(crate) locale: Option<&'a crate::locale::LocaleContext>,
}

pub(crate) fn render_session_picker(
    area: Rect,
    buf: &mut Buffer,
    theme: &Theme,
    mode: SessionPickerRenderMode<'_>,
    ctx: &mut SessionPickerRenderCtx<'_>,
) -> crate::views::picker::PickerHitAreas {
    match mode {
        SessionPickerRenderMode::Fullscreen => {
            crate::views::welcome::render_session_picker_body(area, buf, theme, ctx)
        }
        SessionPickerRenderMode::Modal { window, title } => {
            render_simple_session_picker_modal(area, buf, theme, window, title, ctx)
        }
    }
}

fn empty_hit_areas() -> crate::views::picker::PickerHitAreas {
    crate::views::picker::PickerHitAreas {
        close_button: Rect::default(),
        search_bar: Rect::default(),
        item_rects: vec![],
        entry_indices: vec![],
        tab_rects: vec![],
        filter_rect: None,
    }
}

fn render_simple_session_picker_modal(
    area: Rect,
    buf: &mut Buffer,
    theme: &Theme,
    window: &mut crate::views::modal_window::ModalWindowState,
    title: &str,
    ctx: &mut SessionPickerRenderCtx<'_>,
) -> crate::views::picker::PickerHitAreas {
    use crate::views::modal_window::{ModalSizing, ModalWindowConfig, Shortcut};
    use crate::views::picker::{self, PickerField};

    let text = |id: &str, english: &'static str| {
        ctx.locale
            .map(|locale| locale.named_static_text(id, english))
            .unwrap_or(english)
    };
    let mut shortcuts = vec![
        Shortcut {
            label: text("picker.shortcut.nav", "\u{2191}\u{2193} nav"),
            clickable: false,
            id: 0,
        },
        Shortcut {
            label: text("picker.shortcut.select", "Enter select"),
            clickable: false,
            id: 0,
        },
        Shortcut {
            label: text("picker.shortcut.close", "Esc close"),
            clickable: false,
            id: 0,
        },
    ];
    crate::views::modal_window::push_vim_nav_search_hint_with_locale(
        &mut shortcuts,
        ctx.state.search_active,
        ctx.locale,
    );
    let modal_config = ModalWindowConfig {
        title,
        tabs: None,
        shortcuts: &shortcuts,
        sizing: ModalSizing {
            width_pct: 0.65,
            max_width: 120,
            min_width: 48,
            v_margin: 4,
            h_pad: 2,
            v_pad: 1,
            footer_lines: 2,
        },
        fold_info: None,
    };
    let Some(modal) =
        crate::views::modal_window::render_modal_window(buf, area, window, &modal_config, theme)
    else {
        ctx.state.hit_areas = None;
        return empty_hit_areas();
    };

    let content = modal.content;
    picker::render_picker_search_bar_with_locale(
        buf,
        content.x,
        content.y,
        content.width,
        theme,
        ctx.state,
        ctx.state.search_active,
        true,
        Some(theme.bg_base),
        ctx.locale,
    );
    ctx.state.filter_area = None;
    let separator_y = content.y + 1;
    if separator_y < content.y + content.height {
        picker::render_divider(
            buf,
            modal.inner_x,
            separator_y,
            modal.inner_width,
            theme,
            Some(theme.bg_base),
        );
    }

    let query =
        crate::views::session_picker::effective_filter_query(ctx.state.query(), ctx.entries_query);
    let sessions = ctx.sessions.unwrap_or(&[]);
    let filtered =
        crate::app::app_view::filter_session_entries(ctx.sessions, query, ctx.source_filter);
    let built = crate::views::session_picker::build_session_entry_data_with_locale(
        sessions,
        &filtered,
        ctx.state,
        content.width,
        ctx.locale,
    );
    let fields: Vec<Vec<PickerField<'_>>> = built
        .iter()
        .map(|entry| {
            entry
                .field_data
                .iter()
                .map(|(label, value)| PickerField { label, value })
                .collect()
        })
        .collect();
    let current_repo = crate::views::session_picker::repo_name_from_cwd(&ctx.cwd.to_string_lossy());
    let (entries, non_selectable) = crate::views::session_picker::build_grouped_picker_entries(
        sessions,
        &filtered,
        &built,
        &fields,
        ctx.state,
        Some(current_repo.as_str()),
    );
    let entries_area = Rect {
        x: content.x,
        y: separator_y + 1,
        width: content.width,
        height: content
            .height
            .saturating_sub(separator_y.saturating_add(1).saturating_sub(content.y)),
    };
    let content_hit = picker::render_picker_content_with_scrollbar_x_and_locale(
        buf,
        entries_area,
        theme,
        ctx.state,
        &entries,
        &non_selectable,
        &[],
        Some(theme.bg_base),
        ctx.loading,
        ctx.tick,
        modal.inner_x + modal.inner_width - 1,
        ctx.locale,
    );
    crate::views::picker::PickerHitAreas {
        close_button: Rect::default(),
        search_bar: Rect::new(content.x, content.y, content.width, 1),
        item_rects: content_hit.item_rects,
        entry_indices: content_hit.entry_indices,
        tab_rects: vec![],
        filter_rect: None,
    }
}

#[cfg(test)]
mod locale_tests {
    use super::*;

    #[test]
    fn zh_localization_dashboard_picker_threads_locale_to_modal_chrome() {
        let locale = crate::locale::LocaleContext::new(crate::locale::ResolvedLocale {
            locale: crate::locale::UiLocale::ZhCn,
            source: crate::locale::LocaleSource::Cli,
        });
        for (locale, search_active, title, search, loading, select) in [
            (
                None,
                false,
                "Open session",
                "/ to search",
                "Loading…",
                "Enter select",
            ),
            (
                None,
                true,
                "Open session",
                "search:",
                "Loading…",
                "Enter select",
            ),
            (
                Some(&locale),
                false,
                "打开会话",
                "/ 开始搜索",
                "加载中…",
                "Enter 选择",
            ),
            (
                Some(&locale),
                true,
                "打开会话",
                "搜索：",
                "加载中…",
                "Enter 选择",
            ),
        ] {
            assert_eq!(dashboard_picker_title(locale), title);
            let mut surface = SessionPickerSurface::new(1);
            surface.state.search_active = search_active;
            let area = Rect::new(0, 0, 120, 30);
            let mut buf = Buffer::empty(area);
            let theme = Theme::default();
            let hit = render_session_picker(
                area,
                &mut buf,
                &theme,
                SessionPickerRenderMode::Modal {
                    window: &mut surface.window,
                    title: dashboard_picker_title(locale),
                },
                &mut SessionPickerRenderCtx {
                    state: &mut surface.state,
                    sessions: None,
                    cwd: std::path::Path::new("/repo"),
                    loading: true,
                    pending_hint: None,
                    shortcuts_area: None,
                    content_results: None,
                    content_loading: false,
                    entries_query: None,
                    tick: 0,
                    grouped: true,
                    source_filter: Default::default(),
                    pending_delete: false,
                    chat_mode: false,
                    locale,
                },
            );
            let mut screen = String::new();
            for y in area.y..area.y + area.height {
                for x in area.x..area.x + area.width {
                    if let Some(cell) = buf.cell((x, y)) {
                        screen.push_str(cell.symbol());
                    }
                }
                screen.push('\n');
            }
            let screen = screen.replace(' ', "");
            for label in [title, search, loading, select] {
                assert!(
                    screen.contains(&label.replace(' ', "")),
                    "missing {label:?} in {screen}"
                );
            }
            assert!(hit.search_bar.width > 0);
            assert!(hit.item_rects.is_empty());
        }
    }
}
