// Print stylesheet (plan §3 e4: "…+ print stylesheet so mail + chrome theme
// together"; SPEC §15 export → client-side print-to-PDF).
//
// The chrome (sidebar, list, ribbon, compose, toast) is hidden for print; the
// open reader is promoted to full width and inked black-on-white.
//
// The message body itself lives in the sandboxed iframe and THIS STYLESHEET
// DOES NOT REACH IT. An earlier version of this comment stated as fact that
// the body's print theming comes from `themeCssVars(theme, { forPrint: true })`
// injected into the srcdoc. That injection does not exist: every
// `bodyFrameDoc` call site passes two arguments, so the style-vars parameter is
// always empty, and `themeCssVars` has no runtime caller anywhere in the app.
// The body therefore prints with the sandbox's own hardcoded near-black-on-
// transparent styling, in print and on screen alike, whatever theme is active.
//
// Wiring it is not a one-line fix and is not purely a defect: the call site,
// a producer/consumer variable-name mismatch (`--mw-color-text` vs `--mw-text`),
// the injection ORDER inside `bodyFrameDoc` (the frame's own rules are
// concatenated last at equal specificity and win), and a hardcoded white
// `.reader__frame` background all have to change together — and whether a
// message body should follow the chrome theme at all is a product decision,
// since mail authored for white backgrounds can become unreadable under a dark
// one. Deleting the dead code is a legitimate outcome too. Only the false
// claim is fixed here.

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
