// Each `tests/test_*.rs` file is a separate crate, so scaffolding used by only
// some test crates shows up as dead_code in the others. This is shared by design.
#![allow(dead_code)]

pub mod assertions;
pub mod builder;
pub mod context;
pub mod drivers;
pub mod exports;
pub mod goldens;
pub mod graph_shapes;
pub mod repository_manifest;
pub mod review_schema;
pub mod rule_inputs;
pub mod snapshots;

/// The origin Tauri serves the application from, which is the origin the real
/// capability set grants and the one a test must present to be let through.
/// Windows serves `http://tauri.localhost`; everywhere else it is `tauri://localhost`,
/// and a request from the other platform's origin is refused, reading like a missing
/// plugin (`open_repo not allowed. Plugin not found`).
#[cfg(windows)]
pub const WEBVIEW_URL: &str = "http://tauri.localhost";
#[cfg(not(windows))]
pub const WEBVIEW_URL: &str = "tauri://localhost";

static ISOLATED: std::sync::Once = std::sync::Once::new();

/// Blank libgit2's global, XDG, system and `ProgramData` config search paths, so the
/// operator's git config cannot reach a test repository. Git for Windows turns
/// `core.autocrlf` on in its system config, and libgit2 reads it and ignores
/// `GIT_CONFIG_GLOBAL` and `GIT_CONFIG_NOSYSTEM`, so a checkout wrote CRLF into
/// files the tests compare against LF. Mirrors `trunk_fixtures::isolate`.
///
/// Process-global, and only effective before libgit2 reads a config file, so every
/// helper that creates a repository calls it first.
pub fn isolate() {
    ISOLATED.call_once(|| {
        for level in [
            git2::ConfigLevel::Global,
            git2::ConfigLevel::XDG,
            git2::ConfigLevel::System,
            git2::ConfigLevel::ProgramData,
        ] {
            // SAFETY: libgit2's options are process-global and unsynchronised. This runs
            // once, ahead of any repository the test process opens.
            unsafe { git2::opts::set_search_path(level, "") }
                .expect("blank libgit2's config search path");
        }
    });
}
