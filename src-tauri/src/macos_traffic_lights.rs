//! Centers the macOS traffic-light buttons in Trunk's top bar.
//!
//! Tauri/wry honor the configured inset only at window creation; AppKit then
//! relayouts the buttons back to the default top position on window-state
//! restore, resize, or appearance change, and Tauri 2.10 exposes no runtime
//! setter. So we grow the title-bar container ourselves so the buttons drop to
//! the bar's vertical center, re-applying on the relayout events (see `lib.rs`).
//!
//! The bar is `--topbar-h` (44px) of webview CSS, so its on-screen height scales
//! with the webview zoom; `reposition` takes the zoom factor to stay centered.

use std::ffi::c_void;

use objc2::rc::Retained;
use objc2_app_kit::{NSWindow, NSWindowButton};

/// `--topbar-h` in CSS px; the on-screen bar is this tall times the webview zoom.
const BAR_HEIGHT: f64 = 44.0;
/// Logical x inset of the close button from the window's left edge.
const INSET_X: f64 = 19.0;

/// `ns_window` is the pointer from Tauri's `WebviewWindow`/`Window::ns_window()`;
/// `zoom` is the current webview zoom factor.
pub fn reposition(ns_window: *mut c_void, zoom: f64) {
    let ptr = ns_window.cast::<NSWindow>();
    if ptr.is_null() {
        return;
    }

    // SAFETY: `ptr` is a live NSWindow owned by Tauri for the window's lifetime, and
    // every caller runs on the main thread. `retain` takes a +1 that `Retained`'s Drop
    // balances, leaving Tauri's own reference intact (no over-release).
    let Some(window) = (unsafe { Retained::retain(ptr) }) else {
        return;
    };

    inset(&window, BAR_HEIGHT * zoom);
}

fn inset(window: &NSWindow, bar_height: f64) {
    let (Some(close), Some(miniaturize), Some(zoom)) = (
        window.standardWindowButton(NSWindowButton::CloseButton),
        window.standardWindowButton(NSWindowButton::MiniaturizeButton),
        window.standardWindowButton(NSWindowButton::ZoomButton),
    ) else {
        return;
    };

    // SAFETY: main-thread only; the close button is a live subview of AppKit's
    // title-bar container, reached via the same two superview hops tao uses.
    let Some(title_bar) = (unsafe { close.superview().and_then(|v| v.superview()) }) else {
        return;
    };

    // The buttons keep their default offset within the container, so growing the
    // (top-anchored) container lowers them. To center a button of height `h` in a
    // `bar_height`-tall bar, the container must be `(bar_height + h)/2 + offset`,
    // where `offset` is the button's measured baseline within the container.
    let close_rect = close.frame();
    let button_offset = close_rect.origin.y;
    let mut title_bar_rect = title_bar.frame();
    title_bar_rect.size.height = (bar_height + close_rect.size.height) / 2.0 + button_offset;
    title_bar_rect.origin.y = window.frame().size.height - title_bar_rect.size.height;
    title_bar.setFrame(title_bar_rect);

    let space_between = miniaturize.frame().origin.x - close_rect.origin.x;
    for (i, button) in [close, miniaturize, zoom].into_iter().enumerate() {
        let mut origin = button.frame().origin;
        origin.x = INSET_X + i as f64 * space_between;
        button.setFrameOrigin(origin);
    }
}
