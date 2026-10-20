# Visual baselines: accepted changes

Each entry records a deliberate change to the committed captures, and why it was accepted.
Written by `scripts/visual-accept.sh`; see `docs/visual-regression.md`.

## 2026-09-26

Initial baselines, graph column only, recorded by the TRUNK-100 build from main with the rails drawing. Not yet reviewed by the user.

Changed baselines:

    tests/visual/baselines/graph-lanes__01-behind-only.png
    tests/visual/baselines/graph-lanes__02-local-ahead-no-remote.png
    tests/visual/baselines/graph-lanes__03-detached-old.png
    tests/visual/baselines/graph-lanes__04-tiebreak-upstream-vs-topic.png
    tests/visual/baselines/graph-lanes__05-diverged.png
    tests/visual/baselines/graph-lanes__06-tag-only-chain.png
    tests/visual/baselines/graph-lanes__07-tag-on-unpulled.png
    tests/visual/baselines/graph-lanes__08-stash-on-tip-behind.png
    tests/visual/baselines/graph-lanes__09-branch-point-below-head.png
    tests/visual/baselines/graph-lanes__10-two-remotes.png
    tests/visual/baselines/graph-lanes__11-merge-in-head-chain.png
    tests/visual/baselines/graph-lanes__12-author-vs-committer.png
    tests/visual/baselines/graph-lanes__13-tall-linear.png
    tests/visual/baselines/graph-merges__01-octopus-merge.png
    tests/visual/baselines/graph-merges__02-criss-cross.png
    tests/visual/baselines/graph-merges__03-merge-of-merges.png
    tests/visual/baselines/graph-merges__04-three-topics.png
    tests/visual/baselines/graph-merges__05-sequential-merges.png
    tests/visual/baselines/graph-merges__06-merge-second-parent-newer.png
    tests/visual/baselines/graph-merges__07-fork-sibling-older.png
    tests/visual/baselines/graph-merges__08-fork-sibling-newer.png
    tests/visual/baselines/graph-merges__09-column-saturation.png
    tests/visual/baselines/graph-merges__09-column-saturation__graph-56px.png
    tests/visual/baselines/graph-merges__10-merge-parent-left.png
    tests/visual/baselines/graph-merges__11-fork-in-left.png
    tests/visual/baselines/graph-merges__12-pagination-boundary.png
    tests/visual/baselines/graph-merges__13-freed-column-left.png
    tests/visual/baselines/graph-merges__14-spiral-right-before-left.png

## 2026-09-26

The capture now starts below the column header, whose label GitHub's macOS runner antialiases up to 36 levels away from this Mac (CI run 36204042756). Each new baseline is the old one without its top 28 rows, pixel for pixel, so no drawing changed. Recorded by the TRUNK-100 build without an acceptance direction from the user.

Changed baselines:

    tests/visual/baselines/graph-lanes__01-behind-only.png
    tests/visual/baselines/graph-lanes__02-local-ahead-no-remote.png
    tests/visual/baselines/graph-lanes__03-detached-old.png
    tests/visual/baselines/graph-lanes__04-tiebreak-upstream-vs-topic.png
    tests/visual/baselines/graph-lanes__05-diverged.png
    tests/visual/baselines/graph-lanes__06-tag-only-chain.png
    tests/visual/baselines/graph-lanes__07-tag-on-unpulled.png
    tests/visual/baselines/graph-lanes__08-stash-on-tip-behind.png
    tests/visual/baselines/graph-lanes__09-branch-point-below-head.png
    tests/visual/baselines/graph-lanes__10-two-remotes.png
    tests/visual/baselines/graph-lanes__11-merge-in-head-chain.png
    tests/visual/baselines/graph-lanes__12-author-vs-committer.png
    tests/visual/baselines/graph-lanes__13-tall-linear.png
    tests/visual/baselines/graph-merges__01-octopus-merge.png
    tests/visual/baselines/graph-merges__02-criss-cross.png
    tests/visual/baselines/graph-merges__03-merge-of-merges.png
    tests/visual/baselines/graph-merges__04-three-topics.png
    tests/visual/baselines/graph-merges__05-sequential-merges.png
    tests/visual/baselines/graph-merges__06-merge-second-parent-newer.png
    tests/visual/baselines/graph-merges__07-fork-sibling-older.png
    tests/visual/baselines/graph-merges__08-fork-sibling-newer.png
    tests/visual/baselines/graph-merges__09-column-saturation.png
    tests/visual/baselines/graph-merges__09-column-saturation__graph-56px.png
    tests/visual/baselines/graph-merges__10-merge-parent-left.png
    tests/visual/baselines/graph-merges__11-fork-in-left.png
    tests/visual/baselines/graph-merges__12-pagination-boundary.png
    tests/visual/baselines/graph-merges__13-freed-column-left.png
    tests/visual/baselines/graph-merges__14-spiral-right-before-left.png

## 2026-09-26

Captures now span the commit rows alone, in a window 1000 px tall, so a capture's height follows its repository rather than the chrome around the list or the window. Each new capture equals its old baseline over its rows, the part cut away was background, and 13-tall-linear gains the four rows the 800 px window cut off. Accepted at João's direction on 2026-09-26 (TRUNK-100).

Changed baselines:

    tests/visual/baselines/graph-lanes__01-behind-only.png
    tests/visual/baselines/graph-lanes__02-local-ahead-no-remote.png
    tests/visual/baselines/graph-lanes__03-detached-old.png
    tests/visual/baselines/graph-lanes__04-tiebreak-upstream-vs-topic.png
    tests/visual/baselines/graph-lanes__05-diverged.png
    tests/visual/baselines/graph-lanes__06-tag-only-chain.png
    tests/visual/baselines/graph-lanes__07-tag-on-unpulled.png
    tests/visual/baselines/graph-lanes__08-stash-on-tip-behind.png
    tests/visual/baselines/graph-lanes__09-branch-point-below-head.png
    tests/visual/baselines/graph-lanes__10-two-remotes.png
    tests/visual/baselines/graph-lanes__11-merge-in-head-chain.png
    tests/visual/baselines/graph-lanes__12-author-vs-committer.png
    tests/visual/baselines/graph-lanes__13-tall-linear.png
    tests/visual/baselines/graph-merges__01-octopus-merge.png
    tests/visual/baselines/graph-merges__02-criss-cross.png
    tests/visual/baselines/graph-merges__03-merge-of-merges.png
    tests/visual/baselines/graph-merges__04-three-topics.png
    tests/visual/baselines/graph-merges__05-sequential-merges.png
    tests/visual/baselines/graph-merges__06-merge-second-parent-newer.png
    tests/visual/baselines/graph-merges__07-fork-sibling-older.png
    tests/visual/baselines/graph-merges__08-fork-sibling-newer.png
    tests/visual/baselines/graph-merges__09-column-saturation.png
    tests/visual/baselines/graph-merges__09-column-saturation__graph-56px.png
    tests/visual/baselines/graph-merges__10-merge-parent-left.png
    tests/visual/baselines/graph-merges__11-fork-in-left.png
    tests/visual/baselines/graph-merges__12-pagination-boundary.png
    tests/visual/baselines/graph-merges__13-freed-column-left.png
    tests/visual/baselines/graph-merges__14-spiral-right-before-left.png

## 2026-09-26

New captures of the 21 stash-lanes repositories, the kitchen-sink repository, and 09-column-saturation at 56 px panned 20 px, so a paint break in stash placement beside the WIP row, the WIP marker or a panned graph column fails the suite. Each image equals the candidate João looked at in the TRUNK-289 contact sheet. The 28 existing baselines are unchanged. Accepted at João's direction on 2026-09-27 (TRUNK-289).

Changed baselines:

    tests/visual/baselines/graph-merges__09-column-saturation__graph-56px__panned-20px.png
    tests/visual/baselines/kitchen-sink.png
    tests/visual/baselines/stash-lanes__01-clean-inline.png
    tests/visual/baselines/stash-lanes__02-dirty-tracked.png
    tests/visual/baselines/stash-lanes__03-dirty-untracked.png
    tests/visual/baselines/stash-lanes__04-dirty-staged.png
    tests/visual/baselines/stash-lanes__05-dirty-conflicted.png
    tests/visual/baselines/stash-lanes__06-ignored-stays-inline.png
    tests/visual/baselines/stash-lanes__07-multi-stash-clean.png
    tests/visual/baselines/stash-lanes__08-multi-stash-dirty.png
    tests/visual/baselines/stash-lanes__09-topic-above-parent.png
    tests/visual/baselines/stash-lanes__10-topic-below-parent.png
    tests/visual/baselines/stash-lanes__11-stash-parent-mid-chain.png
    tests/visual/baselines/stash-lanes__12-orphan-stash.png
    tests/visual/baselines/stash-lanes__13-detached-head.png
    tests/visual/baselines/stash-lanes__14-merge-tip.png
    tests/visual/baselines/stash-lanes__15-backdated-stash.png
    tests/visual/baselines/stash-lanes__16-bare-repo.git.png
    tests/visual/baselines/stash-lanes__17-no-stash-dirty.png
    tests/visual/baselines/stash-lanes__18-many-files.png
    tests/visual/baselines/stash-lanes__19-two-backdated.png
    tests/visual/baselines/stash-lanes__20-stash-on-stash.png
    tests/visual/baselines/stash-lanes__21-tagged-stash.png

## 2026-09-27

TRUNK-293: WIP ring drawn as one arc per dash

Changed baselines:

    tests/visual/baselines/kitchen-sink.png
    tests/visual/baselines/stash-lanes__02-dirty-tracked.png
    tests/visual/baselines/stash-lanes__03-dirty-untracked.png
    tests/visual/baselines/stash-lanes__04-dirty-staged.png
    tests/visual/baselines/stash-lanes__05-dirty-conflicted.png
    tests/visual/baselines/stash-lanes__08-multi-stash-dirty.png
    tests/visual/baselines/stash-lanes__17-no-stash-dirty.png
    tests/visual/baselines/stash-lanes__18-many-files.png

## 2026-09-27

TRUNK-291: every stash stops its rails at the square's edge, so a stash with a stash on top shows no rail inside it

Changed baselines:

    tests/visual/baselines/stash-lanes__20-stash-on-stash.png

## 2026-09-28

WIP ring: a whole number of equal dashes and gaps goes round the ring, so the last dash no longer runs on into the first as one double-length dash. Only the WIP marker's pixels change, in the eight captures holding a WIP row. Accepted at João's direction on 2026-09-28.

Changed baselines:

    tests/visual/baselines/kitchen-sink.png
    tests/visual/baselines/stash-lanes__02-dirty-tracked.png
    tests/visual/baselines/stash-lanes__03-dirty-untracked.png
    tests/visual/baselines/stash-lanes__04-dirty-staged.png
    tests/visual/baselines/stash-lanes__05-dirty-conflicted.png
    tests/visual/baselines/stash-lanes__08-multi-stash-dirty.png
    tests/visual/baselines/stash-lanes__17-no-stash-dirty.png
    tests/visual/baselines/stash-lanes__18-many-files.png

## 2026-10-03

first catalog baseline: the design catalog draws the tokens and the Button, LinkButton and ButtonGroup primitives, text masked

Changed baselines:

    tests/visual/baselines/catalog.png

## 2026-10-03

RowAction primitive added to the catalog: five tones at the target and compact sizes, drawn below LinkButton so nothing above moves

Changed baselines:

    tests/visual/baselines/catalog.png

## 2026-10-03

ListOption primitive added to the catalog: a files listbox, a stacked repository list and a pull menu, drawn below RowAction so nothing above moves

Changed baselines:

    tests/visual/baselines/catalog.png

## 2026-10-03

Chip primitive added to the catalog, drawn below ListOption; its four color tokens appear as swatches in the color grid, which grew one row and moved everything below it 32px

Changed baselines:

    tests/visual/baselines/catalog.png

## 2026-10-03

Tab and TabStrip primitives added to the catalog, drawn below Chip as the commit form's Commit, Amend and Stash strip with Amend selected; a dozen antialiased pixels on the first Chip pill's curve and two single pixels on a Button corner also move, and an empty box of the same height in place of the strip moves the same pixels, so they follow the capture height rather than the new primitive

Changed baselines:

    tests/visual/baselines/catalog.png

## 2026-10-03

ToastCard primitive added to the catalog, drawn below Tab as a neutral and a danger card in a column that shrinks to its widest card; the same antialiased pixels on the first Chip pill's curve and one pixel above it move with the capture height, as the Tab entry records

Changed baselines:

    tests/visual/baselines/catalog.png

## 2026-10-03

Button gains a 16px icon-only xs size for the tab close, drawn after the md icon in each Button row, so the disabled and pressed buttons that follow it in every row move 32px right; nothing outside the Button rows moves

Changed baselines:

    tests/visual/baselines/catalog.png

## 2026-10-05

TRUNK-307: catalog sections for Row, the framed Tab, GutterGrip, HitArea and Splitter

Changed baselines:

    tests/visual/baselines/catalog.png

## 2026-10-06

catalog: add the anchored Dialog

Changed baselines:

    tests/visual/baselines/catalog.png

## 2026-10-06

catalog shows Chip's new label variant

Changed baselines:

    tests/visual/baselines/catalog.png

## 2026-10-06

catalog shows a long and a short truncating chip in a narrow row

Changed baselines:

    tests/visual/baselines/catalog.png

## 2026-10-07

catalog shows the review UI's new primitives: xs buttons, destructive row action, entry row, lane chip, Radio, Tag and Keycap

Changed baselines:

    tests/visual/baselines/catalog.png

## 2026-10-08

the catalog shows a button group at each size, xs to lg, now that a group stands at the height of its buttons

Changed baselines:

    tests/visual/baselines/catalog.png

## 2026-10-08

review panel port: the Badge primitive for a short id or count, and the 24px base button size

Changed baselines:

    tests/visual/baselines/catalog.png

## 2026-10-20

first Windows Chromium baselines, recorded on the runner

Changed baselines:

    004c19ccec5291e74a8989a95c493b790e2c2849 *tests/visual/baselines-chromium/stash-lanes__12-orphan-stash.png
    08c6dd217f94adf0d9345d33d676062fc3af0a3b *tests/visual/baselines-chromium/graph-lanes__05-diverged.png
    0df20d8581f2fa7ed3ded82b77cec42f9a5e9730 *tests/visual/baselines-chromium/graph-lanes__08-stash-on-tip-behind.png
    21ef6dc2338aed5d274ce9b3b0ee812264e8f80e *tests/visual/baselines-chromium/graph-merges__05-sequential-merges.png
    24ac3914f2c818d4c24affaad57d9f1b234f095f *tests/visual/baselines-chromium/stash-lanes__17-no-stash-dirty.png
    256e04edb9236dea8bb760f2b396dc5c67284fd8 *tests/visual/baselines-chromium/graph-merges__06-merge-second-parent-newer.png
    2a3abeb38785dbabea1a4f2153ea24d9c357faac *tests/visual/baselines-chromium/stash-lanes__14-merge-tip.png
    2c6f3215b0acfdb1a6dbca0f7d7d55c992d501e1 *tests/visual/baselines-chromium/graph-merges__07-fork-sibling-older.png
    5fd224414ecd042613b7511376676e6ca7b049a6 *tests/visual/baselines-chromium/stash-lanes__08-multi-stash-dirty.png
    64fe05afd38348fe7207ec38903edd032ec84284 *tests/visual/baselines-chromium/graph-lanes__02-local-ahead-no-remote.png
    770cf58ec21ac73fdcc3c57695c30249bc66c7b5 *tests/visual/baselines-chromium/graph-merges__10-merge-parent-left.png
    7a927743284fe9a465e1bb30459b941d21b20de0 *tests/visual/baselines-chromium/stash-lanes__19-two-backdated.png
    7e73a9f55c0d8c3bfeb1e7967a55dc54b8544164 *tests/visual/baselines-chromium/graph-lanes__06-tag-only-chain.png
    7f141e90b65b933eed19661a5d4d60063ee2e054 *tests/visual/baselines-chromium/stash-lanes__10-topic-below-parent.png
    7f441a016d0805c306017ca483246b6050aad8b4 *tests/visual/baselines-chromium/graph-lanes__13-tall-linear.png
    7f9dc80d8d4b8a87954c499a6f488989ca687a1e *tests/visual/baselines-chromium/graph-lanes__10-two-remotes.png
    885b49de93c5b709f3d7084ceaca40b3bbd5b9dc *tests/visual/baselines-chromium/graph-merges__04-three-topics.png
    8fa39151ed8c4fca40616e8b511fad384d42cc51 *tests/visual/baselines-chromium/stash-lanes__11-stash-parent-mid-chain.png
    8fd3d2fbd961b3274a54a6327f65cc58e31f8ef6 *tests/visual/baselines-chromium/graph-lanes__04-tiebreak-upstream-vs-topic.png
    92641d676815ac9b97c011fd1d0c67ac723cbebb *tests/visual/baselines-chromium/stash-lanes__07-multi-stash-clean.png
    93a087090c08536df68b7c96c5b7b4f0b6f271ab *tests/visual/baselines-chromium/stash-lanes__01-clean-inline.png
    a08cc2dde1fe1afc421d6bf0513a906edd2a0232 *tests/visual/baselines-chromium/stash-lanes__13-detached-head.png
    a218097f547196cafa89e4d9ebbf6931eec6c5e6 *tests/visual/baselines-chromium/graph-lanes__03-detached-old.png
    ab11c59c3c4e652ff7abb1374796988d56e8765d *tests/visual/baselines-chromium/stash-lanes__02-dirty-tracked.png
    ab11c59c3c4e652ff7abb1374796988d56e8765d *tests/visual/baselines-chromium/stash-lanes__03-dirty-untracked.png
    ab11c59c3c4e652ff7abb1374796988d56e8765d *tests/visual/baselines-chromium/stash-lanes__04-dirty-staged.png
    ac5f88eea0f2f4ef65cb8e61608ad757408a7bd1 *tests/visual/baselines-chromium/graph-merges__13-freed-column-left.png
    ace53467785b4064765b776d6a4809fdab9bd725 *tests/visual/baselines-chromium/stash-lanes__16-bare-repo.git.png
    b7f03e85e95931cffea86a2043db7e58cfacc121 *tests/visual/baselines-chromium/graph-merges__09-column-saturation__graph-56px__panned-20px.png
    b874d66a123f52588d459561fab901932c28451f *tests/visual/baselines-chromium/stash-lanes__09-topic-above-parent.png
    bbbd78d5f75aeeb73ce927991cf688a262fc100d *tests/visual/baselines-chromium/graph-merges__03-merge-of-merges.png
    c0aeb8a3c0fb55db324d369bd0b6a02069cacdf9 *tests/visual/baselines-chromium/catalog.png
    c238c9bb8ab80b6856776e0dafeb8a53f71c5db7 *tests/visual/baselines-chromium/graph-lanes__07-tag-on-unpulled.png
    c5bcad769b28ef966bbecd0e4aa912f4bc62ab8e *tests/visual/baselines-chromium/stash-lanes__06-ignored-stays-inline.png
    c5bcad769b28ef966bbecd0e4aa912f4bc62ab8e *tests/visual/baselines-chromium/stash-lanes__15-backdated-stash.png
    c5bcad769b28ef966bbecd0e4aa912f4bc62ab8e *tests/visual/baselines-chromium/stash-lanes__21-tagged-stash.png
    cd17a51332b3e9002817b319257181fb3d3cb769 *tests/visual/baselines-chromium/graph-merges__12-pagination-boundary.png
    cde5f26f5007ce8a610f32faefe820a6e861c313 *tests/visual/baselines-chromium/stash-lanes__05-dirty-conflicted.png
    d046b07bf1ebcf34a5aef2b0b8bd80081411fbe5 *tests/visual/baselines-chromium/stash-lanes__20-stash-on-stash.png
    d7f72229e07f7b976465013134e647ec60421cfa *tests/visual/baselines-chromium/graph-lanes__01-behind-only.png
    dd0aca75cc6638d2e5a6b727e0bf48b627dc5465 *tests/visual/baselines-chromium/stash-lanes__18-many-files.png
    dfd4c3ec87ab62cc1e86655af01f676b1291bd80 *tests/visual/baselines-chromium/graph-lanes__12-author-vs-committer.png
    e0062b8b49dceeec2b812e29f94690acdbab79eb *tests/visual/baselines-chromium/graph-merges__08-fork-sibling-newer.png
    e589451f2d414bf5b8184f66b478ba0e00903915 *tests/visual/baselines-chromium/graph-merges__14-spiral-right-before-left.png
    e5e38582758e7e5960d2502a2f987b42ab4510c3 *tests/visual/baselines-chromium/graph-merges__11-fork-in-left.png
    eb3a2e4e859ab5f3709fdd6210257fc6bfb868d7 *tests/visual/baselines-chromium/graph-lanes__09-branch-point-below-head.png
    ebdbbfa2835eece31c94e6cd27468a7e4e37233a *tests/visual/baselines-chromium/kitchen-sink.png
    f0a671c2db8f9e92ae3d03f31b01f93fffb24038 *tests/visual/baselines-chromium/graph-merges__01-octopus-merge.png
    f1eba1dec1e08d6655679f6915ce184479e3bd3b *tests/visual/baselines-chromium/graph-merges__09-column-saturation__graph-56px.png
    fc53e232b96084fd5049bfbdcf611578429e13d6 *tests/visual/baselines-chromium/graph-merges__02-criss-cross.png
    fecdd9df034db868d12e26afc9812806ec12ef97 *tests/visual/baselines-chromium/graph-lanes__11-merge-in-head-chain.png
    ff882d425fd1a25907691265f0aba928b3c980bc *tests/visual/baselines-chromium/graph-merges__09-column-saturation.png
