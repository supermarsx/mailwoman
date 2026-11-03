// Reading-pane layout switch (W3): drives the shell grid off the
// `:root[data-reading-pane]` attribute set by readingPane.ts.
//
// `right` is the default 3-column shell (sidebar | list | reader) and needs no
// override. `bottom` docks the reader under the list; `off` collapses the reader
// column and only shows the reader as a full-width overlay while a message is
// open (`.reader--open`, set by Reader.tsx).
//
// Every override is guarded to the desktop width (≥ 761px) so it never fights the
// narrow-viewport media query in app.css (≤ 760px), which stacks the shell into
// one pane and shows the open reader as a full-height view whatever this setting
// is. The 220px offset matches the shell's fixed sidebar column in app.css.
// Regions are split by the 3px rule in the control border colour (t32), the
// same rule app.css draws between the sidebar, the list and the reader.

import { globalStyle } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

const DESKTOP = 'screen and (min-width: 761px)';

// Keep the already configured skeleton over the opaque frame until its current
// document has parsed. Never show the empty bridge's default canvas in between.
globalStyle('.reader__frame--pending', {visibility:'hidden',pointerEvents:'none'});
globalStyle('.reader__frame-loading', {position:'absolute',inset:0,zIndex:1,overflow:'auto',background:vars.color.surface});
globalStyle('.reader__frame-loading > [role="status"]', {animation:'none'});

globalStyle(':root .reader--open .reader__header', {'@media':{'(max-width: 760px)':{paddingBlock:'4px',paddingInline:'8px',maxBlockSize:'min(42dvh, 17rem)'}}});
globalStyle(':root .reader--open .reader__subject', {'@media':{'(max-width: 760px)':{fontSize:'1rem',lineHeight:'1.25',marginBlockEnd:'4px'}}});
globalStyle(':root .reader--open .reader__close', {'@media':{'(max-width: 760px) and (pointer: fine)':{minHeight:'32px',marginBlockEnd:'4px',paddingBlock:'3px'}}});

// ── bottom: list on top, reader beneath (within the content column) ──────────
globalStyle(':root[data-reading-pane="bottom"] .shell', {
  '@media': { [DESKTOP]: { gridTemplateColumns: '220px minmax(0, 1fr)', gridTemplateRows: 'minmax(0, 1fr) minmax(0, 1fr)' } },
});
globalStyle(':root[data-reading-pane="bottom"] .sidebar', {
  '@media': { [DESKTOP]: { gridColumn: '1', gridRow: '1 / 3' } },
});
globalStyle(':root[data-reading-pane="bottom"] .mail-pane', {
  '@media': { [DESKTOP]: { gridColumn: '2', gridRow: '1', minHeight: 0 } },
});
globalStyle(':root[data-reading-pane="bottom"] .reader', {
  '@media': {
    [DESKTOP]: {
      gridColumn: '2',
      gridRow: '2',
      minHeight: 0,
      borderBlockStart: `${vars.border.heavy} solid ${vars.color.borderStrong}`,
    },
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
      insetBlock: 0,
      insetInlineEnd: 0,
      insetInlineStart: '220px',
      zIndex: 30,
      background: vars.color.bg,
      borderInlineStart: `${vars.border.heavy} solid ${vars.color.borderStrong}`,
    },
  },
});
