# UI loading policy

Loading UI is local. The surrounding interface stays mounted, visible and
interactive while something loads.

## Policy

Full-page loading UI (the `AsyncPending` card, the boot splash) is allowed ONLY for:

1. the first app boot and the authentication handshake;
2. an account or mailbox switch;
3. the first visit of a top-level screen whose chunk has not loaded yet (prefer
   keeping the previous screen visible).

Everything else is local:

- A short inline label with the shared spinner (`InlineLoading`): "Sanitizing…",
  "Loading…", "Fetching message…".
- Or a skeleton limited to the exact region being loaded (message list rows
  before the first list arrives, folder tree rows before the first folder list).
- No backdrop, no blur, no `pointer-events` blocking. Only modal dialogs have a
  scrim, and that scrim belongs to the dialog.
- Selecting a message never re-skeletons the list pane.
- Refetch and revalidation never replace visible content with a skeleton. Keep
  the data and show a subtle busy indicator in that region only.
- Fast work shows nothing: `InlineLoading` draws after 150 ms and
  `useDelayedBusy` keeps an indicator for at least 300 ms once drawn. The text is
  in the DOM from the start as a polite `role="status"`, so assistive technology
  is told even when nothing is drawn.

## Building blocks

- `components/InlineLoading.tsx`: label plus `Spinner`, reduced-motion safe
  (spinner swaps rotation for an opacity pulse), `role="status"`, `aria-live="polite"`.
  Optional `onRetry` turns a wait past the shared 15 s bound into an `AsyncError`.
- `components/useDelayedBusy.ts`: `useDelayedBusy(active, delayMs = 150, minMs = 300)`.
- `components/MessageLoading.tsx`: the reader body wait. The reader header,
  toolbar and attachments render at once; only this label stands in for the body.
  The frame placeholder (`.reader__frame-loading`) is a corner label, not a cover.
- `components/AsyncState.tsx` `AsyncPending`: the full-page card. Referenced only
  from the allowlist in `components/loadingPolicy.test.ts`.

## Guard

`components/loadingPolicy.test.ts` fails when `AsyncPending` is referenced outside the
allowlist, when a loading-named style covers its parent with a background and
`inset: 0`, or when the reader frame wait becomes a cover again.
`components/useDelayedBusy.test.tsx` covers the delay and minimum display with fake
timers and the `InlineLoading` / `MessageLoading` accessibility.

## Inventory

Scope: full = whole page, pane = a column, region = one section or table, inline = a line.
Kind: skel = skeleton, spin = spinner card, label = inline label, cover = overlay.

| Location | Trigger | Was (scope/kind) | Now (scope/kind) | Rationale |
|---|---|---|---|---|
| App.tsx boot Suspense | auth handshake, config | full / skel | full / skel | policy (a), kept |
| App.tsx LazyRoute (via ErrorBoundary.tsx) | first load of Admin / consent chunk | full / skel | full / skel | policy (c), kept |
| screens/Mailbox.tsx module Suspense | first load of a module screen (calendar, contacts, tasks, notes, keys) | pane / spin card | pane / spin card | policy (c), chrome stays; kept |
| screens/Mailbox.tsx folder rows | no folders yet | region / skel | region / skel | folder tree region only; kept |
| components/MessageList.tsx | folder has no rows yet | pane / skel rows | pane / skel rows | first list only; selection never triggers it; kept |
| components/FolderPane.tsx | folder has no rows yet | pane / skel | pane / skel | same; kept |
| components/MessageLoading.tsx ("Sanitizing…") | sanitiser pending | region / panel + skeleton lines, 12 rem min height | inline / label | converted |
| components/DirectReaderFrame.tsx `.reader__frame-loading` | frame not yet presented | region / opaque `inset:0` cover | inline / corner label, `pointer-events:none` | converted |
| components/Reader.tsx attachment policy | policy fetch | region / thumbnail placeholders | region / thumbnail placeholders | one strip; kept |
| screens/Admin/index.tsx | section chunk, capabilities | region / page skeleton | region / heading + label | content region only; header, nav kept |
| screens/Admin/shell/AdminSubtabPanel.tsx | tab chunk | region / spin card | region / label | tab bar stays |
| screens/Admin/shell/BootSkeleton, PageFrame, DataTable | admin data load | region / card or table skeleton | region / label | converted |
| ~45 Admin section forms (Analytics, Branding, Telemetry, Egress, Roles, Sso, ...) | first read | region / spin card | region / label | converted |
| ~10 Settings sections (Identities, Signatures, Sessions, Receipts, Linked/Nextcloud accounts, Mail transfer) | first read | region / spin card | region / label | converted |
| screens/Settings/RetainedSettingsSection.tsx | section chunk | region / spin card | region / label | tab bar and other sections stay |
| components/Outbox.tsx | outbox load | pane / list skel | pane / label | converted |
| modules/contacts, notes, tasks, keys, calendar, nextcloud pickers | list load | pane / list skel | region / label | module chrome stays |
| screens/Attachments.tsx | first load | region / table skel | region / label | converted |
| components/Compose.tsx Suspense | ribbon item and editor chunks | inline | inline | never blanks the composer; kept |
| components/properties/host.tsx, exportDialog/host.tsx | dialog chunk | none (`fallback={null}`) | none | dialog scrims are modal by design |
| viewers/AttachmentViewer.tsx, CalendarAttachmentViewer.tsx | viewer chunk | region / label | region / label | already local |
| screens/Settings/SignatureEditor.tsx | editor chunk | region / label | region / label | already local |
| Modal scrims (compose, calendar, contacts, keys, security panel, sweep, drafts) | dialog open | full / scrim | full / scrim | modal dialogs only |
| `.mw-shell-bg` | theme background image | full / decoration | full / decoration | not a loading state |

Counts before: full 2 (boot, lazy route), pane 8, region 70+, inline 6. After:
full 2, pane 3 (message list, folder pane, module lazy), region 3 skeletons
(folder rows, attachment strip, admin heading), everything else is a label.

Left full-page on purpose: the boot splash and the lazy Admin/consent route
chunks (policy a and c). The module-pane Suspense in Mailbox keeps its card
because the mailbox chrome around it stays interactive.
