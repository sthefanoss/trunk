// The sans face on platforms without San Francisco; macOS resolves `-apple-system`
// first and never asks for it (`--font-sans` in tokens.css names it after SF Pro).
//
// Imported from script rather than `@import`ed in app.css: Tailwind inlines an
// `@import` itself and leaves the `url(./files/...)` paths unrebased, so the font
// files were never bundled and the face never loaded.
import "@fontsource-variable/inter/wght.css";
