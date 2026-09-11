//! `/feedback <text>`: the drain-time predraft write; the drain and the modal send share [`select_feedback_images`], so a dropped image never drops the report.

use std::path::Path;

use crate::app::agent_view::{AgentView, PromptInputMode, PromptMode};
use xai_grok_feedback::{
    DraftImage, DraftImagePolicy, FeedbackDraftId, FeedbackDraftStore, FeedbackStoreError,
    derive_title, read_draft_images, write_draft_images,
};
use xai_grok_shell::session::{
    MAX_FEEDBACK_IMAGE_BYTES, MAX_FEEDBACK_IMAGE_TOTAL_BYTES, MAX_FEEDBACK_IMAGES,
    feedback_image_extension,
};

/// Pushed once per requeued row: a bare Enter on an empty composer drains nothing, so the gesture is a real send.
pub(super) const FEEDBACK_STORE_BUSY_NOTICE: &str = "Could not save the feedback draft right now (another process has it open); the report stays queued. Send your next message to retry.";

/// Applies the send-time image policy in order; `None` is an image whose bytes could not be read or decoded.
/// Returns the accepted indices plus the user notice for anything dropped.
pub(crate) fn select_feedback_images(
    images: &[Option<(Vec<u8>, String)>],
) -> (Vec<usize>, Option<String>) {
    select_feedback_images_with_locale(images, &crate::locale::LocaleContext::default())
}

/// Same attachment policy in every locale; only the human-readable notice changes.
pub(crate) fn select_feedback_images_with_locale(
    images: &[Option<(Vec<u8>, String)>],
    locale: &crate::locale::LocaleContext,
) -> (Vec<usize>, Option<String>) {
    let mut accepted = Vec::new();
    let mut over_count = 0usize;
    let mut unsupported = 0usize;
    let mut too_large = 0usize;
    let mut unreadable = 0usize;
    let mut total_bytes = 0usize;
    for (index, image) in images.iter().enumerate() {
        let Some((bytes, mime_type)) = image.as_ref().filter(|(bytes, _)| !bytes.is_empty()) else {
            unreadable += 1;
            continue;
        };
        if accepted.len() >= MAX_FEEDBACK_IMAGES {
            over_count += 1;
            continue;
        }
        if feedback_image_extension(mime_type).is_none() {
            unsupported += 1;
            continue;
        }
        if bytes.len() > MAX_FEEDBACK_IMAGE_BYTES
            || total_bytes + bytes.len() > MAX_FEEDBACK_IMAGE_TOTAL_BYTES
        {
            too_large += 1;
            continue;
        }
        total_bytes += bytes.len();
        accepted.push(index);
    }
    let dropped = over_count + unsupported + too_large + unreadable;
    let notice = (dropped > 0).then(|| {
        const MIB: usize = 1024 * 1024;
        let mut reasons = Vec::new();
        if over_count > 0 {
            reasons.push(
                locale
                    .named_text(
                        "feedback.images.over_limit",
                        "{count} over the {limit}-image limit",
                    )
                    .replace("{count}", &over_count.to_string())
                    .replace("{limit}", &MAX_FEEDBACK_IMAGES.to_string()),
            );
        }
        if unsupported > 0 {
            reasons.push(
                locale
                    .named_text(
                        "feedback.images.unsupported",
                        "{count} in a format feedback can't carry (PNG, JPEG, or GIF only)",
                    )
                    .replace("{count}", &unsupported.to_string()),
            );
        }
        if too_large > 0 {
            reasons.push(
                locale
                    .named_text(
                        "feedback.images.too_large",
                        "{count} over the size limit ({each} MB each, {total} MB combined)",
                    )
                    .replace("{count}", &too_large.to_string())
                    .replace("{each}", &(MAX_FEEDBACK_IMAGE_BYTES / MIB).to_string())
                    .replace(
                        "{total}",
                        &(MAX_FEEDBACK_IMAGE_TOTAL_BYTES / MIB).to_string(),
                    ),
            );
        }
        if unreadable > 0 {
            reasons.push(
                locale
                    .named_text("feedback.images.unreadable", "{count} unreadable")
                    .replace("{count}", &unreadable.to_string()),
            );
        }
        let plural = if dropped == 1 { "" } else { "s" };
        locale
            .named_text(
                "feedback.images.dropped",
                "Dropped {dropped} image{plural} from the feedback: {reasons}.",
            )
            .replace("{dropped}", &dropped.to_string())
            .replace("{plural}", plural)
            .replace("{reasons}", &reasons.join(", "))
    });
    (accepted, notice)
}

#[derive(Debug)]
pub(super) struct InlineDraftSaved {
    pub(super) draft_id: FeedbackDraftId,
    /// Images dropped by policy or lost to a write failure; the text is saved either way.
    pub(super) notice: Option<String>,
}

#[derive(Debug)]
pub(super) enum InlineDraftSaveError {
    /// Another process holds the store lock; the row can be retried unchanged.
    Busy,
    Failed(String),
}

const DRAFT_IMAGE_POLICY: DraftImagePolicy = DraftImagePolicy {
    max_images: MAX_FEEDBACK_IMAGES,
    max_image_bytes: MAX_FEEDBACK_IMAGE_BYTES,
    extension_for_mime: feedback_image_extension,
};

/// `images` are the row's wire images decoded to `(bytes, mime_type)`, `None` where decoding failed.
/// Images are written after the JSON commit so they are keyed by a real id; a write failure downgrades to a notice instead of deleting the draft.
pub(super) fn save_inline_feedback_draft(
    session_dir: Option<&Path>,
    user_text: &str,
    images: &[Option<(Vec<u8>, String)>],
) -> Result<InlineDraftSaved, InlineDraftSaveError> {
    let Some(session_dir) = session_dir else {
        return Err(InlineDraftSaveError::Failed("No active session".to_owned()));
    };
    let (accepted, mut notice) = select_feedback_images(images);
    let draft = FeedbackDraftStore::new(session_dir)
        .append_predraft(&derive_title(user_text), user_text)
        .map_err(|error| match error {
            FeedbackStoreError::Busy => InlineDraftSaveError::Busy,
            FeedbackStoreError::InvalidSessionDirectory { .. } => {
                InlineDraftSaveError::Failed("No active session".to_owned())
            }
            other => InlineDraftSaveError::Failed(format!(
                "Could not save the local feedback draft: {other}"
            )),
        })?;
    let accepted: Vec<DraftImage> = accepted
        .iter()
        .filter_map(|&index| images[index].as_ref())
        .map(|(bytes, mime_type)| DraftImage {
            bytes: bytes.clone(),
            mime_type: mime_type.clone(),
        })
        .collect();
    if let Err(error) = write_draft_images(session_dir, &draft.id, &accepted, DRAFT_IMAGE_POLICY) {
        let error = format!("Could not save feedback draft images: {error}");
        notice = Some(match notice {
            Some(notice) => format!("{notice} {error}"),
            None => error,
        });
    }
    Ok(InlineDraftSaved {
        draft_id: draft.id,
        notice,
    })
}

/// Puts a report whose draft could not be saved back where the user can recover it and returns the notice to show.
/// The composer is spliced through [`crate::views::prompt_widget::PromptWidget::prepend_text`] (never `set_text`, which drops its chips). While it holds a queued row's edit buffer or a non-prompt input mode (`!`/`#`) the composer is left alone and the report is echoed in the notice instead.
pub(super) fn restore_inline_feedback_report(
    agent: &mut AgentView,
    report: &str,
    images: Vec<(Vec<u8>, String)>,
    failure: String,
) -> String {
    let composer_is_taken = matches!(agent.prompt_mode, PromptMode::EditingQueued { .. })
        || agent.prompt_input_mode != PromptInputMode::Normal;
    if composer_is_taken {
        let images_note = match images.len() {
            0 => String::new(),
            1 => " (its image was dropped)".to_owned(),
            count => format!(" (its {count} images were dropped)"),
        };
        return format!("{failure}\nNot sent: {report}{images_note}");
    }
    let separator = if agent.prompt.text().is_empty() {
        ""
    } else {
        "\n"
    };
    // The chips go after the report on its own line, so leave room for them before the separator.
    let chip_gap = if images.is_empty() { "" } else { " " };
    agent
        .prompt
        .prepend_text(&format!("{report}{chip_gap}{separator}"));
    agent.prompt.set_cursor(report.len() + chip_gap.len());
    let mut refused = 0usize;
    for (data, mime_type) in images {
        let image = crate::prompt_images::from_clipboard_data(&crate::clipboard::ImageData {
            data,
            mime_type,
        });
        if agent.prompt.insert_image(image).is_err() {
            refused += 1;
        }
    }
    if refused == 0 {
        return failure;
    }
    let plural = if refused == 1 { "" } else { "s" };
    format!("{failure}; {refused} image{plural} could not be restored.")
}

pub(super) fn attach_saved_draft_images(
    modal: &mut crate::views::feedback_modal::FeedbackModalState,
    session_dir: &Path,
    draft_id: &FeedbackDraftId,
) {
    for image in read_feedback_draft_images(session_dir, draft_id) {
        let _ = modal.insert_image(image);
    }
}

pub(super) fn read_feedback_draft_images(
    session_dir: &Path,
    draft_id: &FeedbackDraftId,
) -> Vec<crate::prompt_images::PastedImage> {
    read_draft_images(session_dir, draft_id, DRAFT_IMAGE_POLICY)
        .into_iter()
        .map(|image| {
            crate::prompt_images::from_clipboard_data(&crate::clipboard::ImageData {
                data: image.bytes,
                mime_type: image.mime_type,
            })
        })
        .collect()
}

#[cfg(test)]
mod localization_tests {
    use super::*;
    use crate::locale::{LocaleContext, LocaleSource, ResolvedLocale, UiLocale};

    fn chinese() -> LocaleContext {
        LocaleContext::new(ResolvedLocale {
            locale: UiLocale::ZhCn,
            source: LocaleSource::Cli,
        })
    }

    #[test]
    fn zh_localization_feedback_image_policy_preserves_selection_and_english_notice() {
        let images = vec![
            Some((vec![1, 2, 3], "image/png".to_owned())),
            Some((vec![4], "image/svg+xml".to_owned())),
            None,
        ];
        let (english_indices, english_notice) = select_feedback_images(&images);
        let (chinese_indices, chinese_notice) =
            select_feedback_images_with_locale(&images, &chinese());
        assert_eq!(english_indices, vec![0]);
        assert_eq!(chinese_indices, english_indices);
        assert_eq!(
            english_notice.as_deref(),
            Some(
                "Dropped 2 images from the feedback: 1 in a format feedback can't carry (PNG, JPEG, or GIF only), 1 unreadable."
            )
        );
        let notice = chinese_notice.expect("rejected attachments have a notice");
        assert!(notice.contains("已从反馈中移除 2 张图片"));
        assert!(notice.contains("PNG、JPEG 或 GIF"));
        assert!(notice.contains("1 张无法读取"));
        assert!(!notice.contains('{'), "all placeholders must be expanded");
    }

    #[test]
    fn zh_localization_feedback_image_policy_keeps_limits_and_empty_success() {
        let locale = chinese();
        let valid = Some((vec![1], "image/png".to_owned()));
        let images = vec![valid.clone(); MAX_FEEDBACK_IMAGES];
        assert_eq!(select_feedback_images_with_locale(&images, &locale).1, None);

        let mut over_limit = images;
        over_limit.push(valid);
        let (indices, notice) = select_feedback_images_with_locale(&over_limit, &locale);
        assert_eq!(indices.len(), MAX_FEEDBACK_IMAGES);
        assert_eq!(indices, select_feedback_images(&over_limit).0);
        let notice = notice.unwrap();
        assert!(notice.contains(&format!("1 张超出 {MAX_FEEDBACK_IMAGES} 张图片上限")));
        assert!(!notice.contains('{'));
        assert!(
            select_feedback_images(&over_limit)
                .1
                .unwrap()
                .starts_with("Dropped 1 image from")
        );

        let oversized = vec![Some((
            vec![0; MAX_FEEDBACK_IMAGE_BYTES + 1],
            "image/png".to_owned(),
        ))];
        let (indices, notice) = select_feedback_images_with_locale(&oversized, &locale);
        assert!(indices.is_empty());
        let notice = notice.unwrap();
        assert!(notice.contains("1 张超过大小限制"));
        assert!(!notice.contains('{'));
    }
}
