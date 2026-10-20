# Inter where San Francisco is missing

## Decision

`--font-sans` is `-apple-system, BlinkMacSystemFont, "SF Pro", "Inter Variable", "Helvetica Neue", sans-serif`,
and the application bundles Inter (`@fontsource-variable/inter`, OFL-1.1) so that `"Inter Variable"`
resolves on every platform. macOS resolves `-apple-system` first and never asks for it, so its look
is unchanged. `--font-mono` is unchanged and no mono face is bundled.

## Why

Every name before `sans-serif` in the old stack is Apple's. On Windows none exist, so the whole
interface drew in the generic `sans-serif`, which is Arial there: not Windows's own UI font, and
not close to the macOS design the type scale (`--text-*`) was drawn for.

San Francisco cannot be the answer on Windows. Apple licenses it for software on Apple platforms,
and shipping the files in this repository is not permitted. Inter is the open face closest to it.
It is not metric-compatible, so Windows will never match a macOS baseline pixel for pixel; a
Windows baseline set is compared only with itself.

A bundled face has the same metrics on every Windows machine and every CI runner. That is what
lets Windows visual baselines be recorded once and mean something, where the system fallback
would differ between a developer's machine and a runner.

## What was weighed

- **Inter first everywhere** would make the platforms match, and change the macOS interface to do it.
- **A face injected only into the visual harness** gives stable baselines but no change to what
  Windows users see, and the baselines would not show what the application really draws there.
- **Bundling JetBrains Mono**, which `--font-mono` already names first, would change macOS for
  every user who does not have it installed, from SF Mono to JetBrains Mono, and move the width of
  every SHA and chip. Windows falls to Consolas, which every Windows machine has.

## How it is wired, and the trap in it

`src/fonts.ts` imports `@fontsource-variable/inter/wght.css`, and `main.ts` and the two visual
harness pages import `fonts.ts`. It must stay a script import. The same stylesheet `@import`ed from
`app.css` builds cleanly and ships nothing: Tailwind inlines an `@import` itself and leaves the
`url(./files/...)` paths unrebased, so the font files were never bundled and the face never loaded.
`dist/assets` should hold seven `inter-*-wght-normal-*.woff2` files, 218 KB together, and the CSP's
`font-src 'self' data:` already allows them. The guard on `font-family` in project CSS (see
`design-system.md`) is why no `@font-face` is written by hand.

`NOTICE` carries the OFL attribution. The installer itself does not carry the license text, which
is worth settling before binaries are distributed.
