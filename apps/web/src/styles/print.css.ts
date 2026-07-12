// Print stylesheet (plan §3 e4: "…+ print stylesheet so mail + chrome theme
// together"; SPEC §15 export → client-side print-to-PDF).
//
// The chrome (sidebar, list, ribbon, compose, toast) is hidden for print; the
// open reader is promoted to full width and inked black-on-white. The message
// body itself lives in the sandboxed iframe — its print theming comes from
// `themeCssVars(theme, { forPrint: true })`, injected into the srcdoc by e7.

import { globalStyle } from '@vanilla-extract/css';
import { vars } from '../theme/contract.css.ts';

globalStyle('body', {
  '@media': {
    print: {
      background: '#ffffff',
      color: '#000000',
      backgroundImage: 'none',
    },
  },
});

// Hide non-message chrome when printing.
for (const sel of ['.sidebar', '.list', '.ribbon', '.compose__backdrop', '.toast']) {
  globalStyle(sel, { '@media': { print: { display: 'none' } } });
}

// Promote the reader pane to the full page.
globalStyle('.shell', {
  '@media': { print: { display: 'block', height: 'auto' } },
});
globalStyle('.reader', {
  '@media': { print: { display: 'block', overflow: 'visible' } },
});
globalStyle('.reader__frame', {
  '@media': {
    print: {
      minHeight: '80vh',
      background: '#ffffff',
      border: 'none',
    },
  },
});

// Keep the header readable and use the reading font in print.
globalStyle('.reader__header', {
  '@media': {
    print: {
      borderBottom: `1px solid ${vars.color.border}`,
      fontFamily: vars.font.reading,
    },
  },
});
globalStyle('.reader__close', { '@media': { print: { display: 'none' } } });
