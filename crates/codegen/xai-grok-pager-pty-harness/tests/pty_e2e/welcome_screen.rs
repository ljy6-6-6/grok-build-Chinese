// Per-test-case module for the `pty_e2e` integration test crate.
#[allow(unused_imports)]
use super::common::*;

/// 1. **Welcome screen.**
/// The pager boots and draws its welcome screen within the timeout.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn welcome_screen() {
    let content = ContentController::start().await.expect("start content");

    let binary = pager_binary().expect("resolve pager binary");
    let mut harness =
        PtyHarness::spawn_with_content(&binary, DEFAULT_ROWS, DEFAULT_COLS, &content, &[])
            .expect("spawn pager");

    harness
        .wait_for_text(WELCOME_SCREEN_SENTINEL, WELCOME_TIMEOUT)
        .expect("welcome text");

    harness.quit().expect("clean quit");
}

/// The production locale resolver and full PTY render path honor a Chinese
/// override even though the shared upstream harness defaults to English.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore]
async fn welcome_screen_zh_cn_override() {
    let content = ContentController::start().await.expect("start content");

    let binary = pager_binary().expect("resolve pager binary");
    let mut harness = PtyHarness::spawn_with_content_env(
        &binary,
        DEFAULT_ROWS,
        DEFAULT_COLS,
        &content,
        &[],
        &[("GROK_ZH_LOCALE", "zh-CN")],
    )
    .expect("spawn Chinese pager");

    harness
        .wait_for_text("退出", WELCOME_TIMEOUT)
        .expect("Chinese welcome text");
    assert!(harness.contains_text("新建工作树"));

    harness.quit().expect("clean quit");
}
