---
status: resolved
trigger: "SHA column left of Message (should be right); squash arrow renders in validation error row"
created: 2026-03-23T00:00:00Z
updated: 2026-03-23T00:00:00Z
---

## Current Focus

hypothesis: Both root causes confirmed
test: Template analysis
expecting: N/A
next_action: Return diagnosis

## Symptoms

expected: Column order Action, Message, SHA, Author, Date; squash arrow next to commit dot
actual: SHA is left of Message; squash arrow appears in validation error row
errors: none (visual layout bugs)
reproduction: Open rebase editor with commits; set all to squash to see arrow in error row
started: unknown

## Eliminated

## Evidence

- timestamp: 2026-03-23T00:01:00Z
  checked: Header template column order (lines 379-417)
  found: Order is Action, SHA, Message, Author, Date — SHA comes before Message
  implication: SHA and Message are swapped vs desired order

- timestamp: 2026-03-23T00:01:00Z
  checked: Data row column order (lines 438-490)
  found: Order is Action, SHA, Message, Author, Date — same wrong order as header
  implication: Both header and data rows have SHA before Message

- timestamp: 2026-03-23T00:02:00Z
  checked: squash arrow positioning context
  found: Arrow is position:absolute with bottom:-4px inside rebase-row-wrapper (position:relative). The wrapper contains BOTH the rebase-row AND the validation-error div. When error is present, the wrapper grows taller, and bottom:-4px positions the arrow relative to the wrapper bottom — which is now the bottom of the error div, not the row.
  implication: Arrow lands visually inside the error row instead of next to the commit dot

## Resolution

root_cause: |
  Issue 1: In the template (both header lines 383-397 and data row lines 457-470), the SHA column block is placed BEFORE the Message column block. The desired order is Action, Message, SHA, Author, Date but the code renders Action, SHA, Message, Author, Date.

  Issue 2: The squash arrow span (line 425) is position:absolute with bottom:-4px inside rebase-row-wrapper (position:relative, line 845). The wrapper encloses both the commit row div AND the validation error div (lines 494-498). When a validation error is present (e.g. "Cannot squash the first commit"), the wrapper's height grows to include the error div. Since the arrow is positioned from the bottom of its containing block (the wrapper), it shifts down into the error row area instead of staying anchored near the commit dot.
fix:
verification:
files_changed: []
