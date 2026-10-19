// Read from navigator.platform alone: every real webview sets it (`MacIntel`,
// `Win32`), and jsdom leaves it empty. The user agent would not do, since
// jsdom's names the host OS and a test run on Windows would take itself for the
// Windows app.
const platform = typeof navigator === "undefined" ? "" : navigator.platform;

/** Whether the webview runs on macOS, where the window's traffic lights float over the top bar. */
export const isMac = /Mac/i.test(platform);

/** Whether the webview runs on Windows, where the top bar replaces the native frame. */
export const isWindows = /Win/i.test(platform);

/** Width the top bar leaves free for the traffic lights; other platforms draw a native title bar above it. */
export const TRAFFIC_LIGHT_INSET = isMac ? 78 : 0;
