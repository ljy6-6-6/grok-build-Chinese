//! Fixed feedback UI copy only; never translate user-authored text or serialized taxonomy.
use crate::locale::LocaleContext;
pub(crate) fn text(locale: &LocaleContext, english: &'static str) -> &'static str {
    let key = match english {
        "Feedback" => "feedback.modal.feedback",
        "Write" => "feedback.modal.write",
        "Drafts" => "feedback.modal.drafts",
        "Type" => "feedback.modal.type",
        "Task" => "feedback.modal.task",
        "Failure" => "feedback.modal.failure",
        "(choose)" => "feedback.modal.choose",
        "Bug" => "feedback.modal.bug",
        "Idea" => "feedback.modal.idea",
        "Missing capability" => "feedback.modal.missing_capability",
        "Code edit" => "feedback.modal.code_edit",
        "Debug" => "feedback.modal.debug",
        "Explain" => "feedback.modal.explain",
        "Plan" => "feedback.modal.plan",
        "Shell" => "feedback.modal.shell",
        "Search" => "feedback.modal.search",
        "Review" => "feedback.modal.review",
        "Other" => "feedback.modal.other",
        "Unclassified" => "feedback.modal.unclassified",
        "Overeager" => "feedback.modal.overeager",
        "Stopping early" => "feedback.modal.stopping_early",
        "Unwanted scope" => "feedback.modal.unwanted_scope",
        "Didn't ask for help" => "feedback.modal.didn_t_ask_for_help",
        "Excessive questions" => "feedback.modal.excessive_questions",
        "Subagent overspawn" => "feedback.modal.subagent_overspawn",
        "Over correction" => "feedback.modal.over_correction",
        "Instruction following" => "feedback.modal.instruction_following",
        "Overconfidence and hallucination" => "feedback.modal.overconfidence_and_hallucination",
        "Code quality" => "feedback.modal.code_quality",
        "Destructive actions" => "feedback.modal.destructive_actions",
        "Context and memory" => "feedback.modal.context_and_memory",
        "Repetition and looping" => "feedback.modal.repetition_and_looping",
        "Model regression" => "feedback.modal.model_regression",
        "Dispute or decline" => "feedback.modal.dispute_or_decline",
        "Tone or preachiness" => "feedback.modal.tone_or_preachiness",
        "Unclear output" => "feedback.modal.unclear_output",
        "↑↓ move" => "feedback.modal.move",
        "type filter" => "feedback.modal.type_filter",
        "Enter select" => "feedback.modal.enter_select",
        "Esc back" => "feedback.modal.esc_back",
        "↑↓/j k move" => "feedback.modal.j_k_move",
        "/ search" => "feedback.modal.shortcut_search",
        "Enter open" => "feedback.modal.enter_open",
        "d delete" => "feedback.modal.d_delete",
        "Enter submit" => "feedback.modal.enter_submit",
        "Esc cancel" => "feedback.modal.esc_cancel",
        "Esc close" => "feedback.modal.esc_close",
        "↑/Tab labels" => "feedback.modal.tab_labels",
        "←→ value" => "feedback.modal.value",
        "Enter edit" => "feedback.modal.enter_edit",
        "Tab done" => "feedback.modal.tab_done",
        "Discard the current Write composition and open the selected draft?" => {
            "feedback.modal.discard_the_current_write_composition_and_open_the_selected_draft"
        }
        "y discard  |  n cancel" => "feedback.modal.y_discard_n_cancel",
        "Delete the stored recovery copy? Your current edits will remain." => {
            "feedback.modal.delete_the_stored_recovery_copy_your_current_edits_will_remain"
        }
        "Delete this feedback draft?" => "feedback.modal.delete_this_feedback_draft",
        "Deleting…" => "feedback.modal.deleting",
        "y delete  |  n cancel" => "feedback.modal.y_delete_n_cancel",
        "Open Drafts to load saved feedback." => {
            "feedback.modal.open_drafts_to_load_saved_feedback"
        }
        "Loading drafts…" => "feedback.modal.loading_drafts",
        "No drafts." => "feedback.modal.no_drafts",
        "No matching drafts." => "feedback.modal.no_matching_drafts",
        "Tell us what happened" => "feedback.modal.tell_us_what_happened",
        "Send this session's trace" => "feedback.modal.send_this_session_s_trace",
        "No, just the feedback" => "feedback.modal.no_just_the_feedback",
        "No, and don't ask again" => "feedback.modal.no_and_don_t_ask_again",
        "One archive of this session is sent with this report only. Nothing is turned on for future sessions." => {
            "feedback.modal.one_archive_of_this_session_is_sent_with_this_report_only_nothing_is_turned_on_for_future_sessions"
        }
        "Attach this session's trace to help us debug this bug?" => {
            "feedback.modal.attach_this_session_s_trace_to_help_us_debug_this_bug"
        }
        "Attach this session's trace to give this idea context?" => {
            "feedback.modal.attach_this_session_s_trace_to_give_this_idea_context"
        }
        "Attach this session's trace to show what was missing?" => {
            "feedback.modal.attach_this_session_s_trace_to_show_what_was_missing"
        }
        "Attach this session's trace to your feedback?" => {
            "feedback.modal.attach_this_session_s_trace_to_your_feedback"
        }
        "Add feedback text or an image before sending." => {
            "feedback.modal.add_feedback_text_or_an_image_before_sending"
        }
        "Sending draft…" => "feedback.modal.sending_draft",
        "Feedback was sent, but the stored draft could not be deleted. Delete it manually; do not resend." => {
            "feedback.modal.feedback_was_sent_but_the_stored_draft_could_not_be_deleted_delete_it_manually_do_not_resend"
        }
        "The remote outcome is unknown. The latest text was copied to the clipboard. Saving it back to this draft; close and do not resend." => {
            "feedback.modal.the_remote_outcome_is_unknown_the_latest_text_was_copied_to_the_clipboard_saving_it_back_to_this_draft_close_and_do_not_resend"
        }
        "The remote outcome is unknown. The latest text was copied to the clipboard. Close and do not resend." => {
            "feedback.modal.the_remote_outcome_is_unknown_the_latest_text_was_copied_to_the_clipboard_close_and_do_not_resend"
        }
        "The remote outcome is unknown. The latest text was saved to this draft and copied to the clipboard. Close and do not resend." => {
            "feedback.modal.the_remote_outcome_is_unknown_the_latest_text_was_saved_to_this_draft_and_copied_to_the_clipboard_close_and_do_not_resend"
        }
        "The remote outcome is unknown. The latest text was copied to the clipboard, but it could not be saved to the draft. Close and do not resend." => {
            "feedback.modal.the_remote_outcome_is_unknown_the_latest_text_was_copied_to_the_clipboard_but_it_could_not_be_saved_to_the_draft_close_and_do_not_resend"
        }
        "Couldn't restore one feedback image; the original file was kept." => {
            "feedback.modal.couldn_t_restore_one_feedback_image_the_original_file_was_kept"
        }
        "Feedback closed because the turn-cancel prompt needs an answer." => {
            "feedback.modal.feedback_closed_because_the_turn_cancel_prompt_needs_an_answer"
        }
        "Feedback closed because a plan is ready for approval." => {
            "feedback.modal.feedback_closed_because_a_plan_is_ready_for_approval"
        }
        "Feedback closed because a permission request needs an answer." => {
            "feedback.modal.feedback_closed_because_a_permission_request_needs_an_answer"
        }
        "Feedback closed because the agent asked a question." => {
            "feedback.modal.feedback_closed_because_the_agent_asked_a_question"
        }
        "Feedback closed because another prompt needs an answer." => {
            "feedback.modal.feedback_closed_because_another_prompt_needs_an_answer"
        }
        "Feedback closed because a tool needs your input." => {
            "feedback.modal.feedback_closed_because_a_tool_needs_your_input"
        }
        "Feedback closed because a hook blocked the prompt." => {
            "feedback.modal.feedback_closed_because_a_hook_blocked_the_prompt"
        }
        _ => return english,
    };
    locale.named_static_text(key, english)
}
