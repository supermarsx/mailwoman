// Reading-pane layout switch (W3): drives the shell grid off the
// `:root[data-reading-pane]` attribute set by readingPane.ts.
//
// `right` is the default 3-column shell (sidebar | list | reader) and needs no
// override. `bottom` docks the reader under the list; `off` collapses the reader
// column and only shows the reader as a full-width overlay while a message is
// open (`.reader--open`, set by Reader.tsx).
//
// Every override is guarded to the desktop width (≥ 761px) so it never fights the
// existing mobile media query in app.css (which single-columns the shell and
// hides the reader at ≤ 760px). The 220px offset matches the shell's fixed
// sidebar column in app.css.

import { globalStyle } from '@vanilla-extract/css';

const DESKTOP = 'screen and (min-width: 761px)';

// ── bottom: list on top, reader beneath (within the content column) ──────────
globalStyle(':root[data-reading-pane="bottom"] .shell', {
  '@media': { [DESKTOP]: { gridTemplateColumns: '220px 1fr', gridTemplateRows: '1fr 1fr' } },
});
globalStyle(':root[data-reading-pane="bottom"] .sidebar', {
  '@media': { [DESKTOP]: { gridColumn: '1', gridRow: '1 / 3' } },
});
globalStyle(':root[data-reading-pane="bottom"] .mail-pane', {
  '@media': { [DESKTOP]: { gridColumn: '2', gridRow: '1', minHeight: 0 } },
});
globalStyle(':root[data-reading-pane="bottom"] .reader', {
  '@media': {
    [DESKTOP]: { gridColumn: '2', gridRow: '2', minHeight: 0, borderTop: '1px solid var(--border)' },
  },
});

// ── off: no persistent reader; list takes the whole content column ───────────
globalStyle(':root[data-reading-pane="off"] .shell', {
  '@media': { [DESKTOP]: { gridTemplateColumns: '220px 1fr' } },
});
globalStyle(':root[data-reading-pane="off"] .mail-pane', {
  '@media': { [DESKTOP]: { gridColumn: '2' } },
});
// Hidden when nothing is open …
globalStyle(':root[data-reading-pane="off"] .reader:not(.reader--open)', {
  '@media': { [DESKTOP]: { display: 'none' } },
});
// … and a full-content overlay while reading (its own back button closes it).
globalStyle(':root[data-reading-pane="off"] .reader.reader--open', {
  '@media': {
    [DESKTOP]: {
      position: 'fixed',
      top: 0,
      right: 0,
      bottom: 0,
      left: '220px',
      zIndex: 30,
      background: 'var(--bg)',
      borderLeft: '1px solid var(--border)',
    },
  },
});
