import { render } from 'solid-js/web';
import { App } from './App.tsx';
import { LocaleProvider } from './i18n/index.ts';
import './styles/app.css';

// Trusted Types default policy (SPEC §7.4, 26.17). The shell ships under
// `require-trusted-types-for 'script'`, so every string that reaches a DOM
// injection sink (`Element.innerHTML`, `iframe.srcdoc`) must be produced by a
// Trusted Types policy or the browser throws. Register a `default` policy BEFORE
// `render()` so it is in place for Solid's very first `template().innerHTML` boot
// write — without it the SPA fails to render at boot under the enforced CSP.
//
// The policy passes the HTML through unchanged: every sink in this app is already
// app-controlled or sanitized upstream — Solid compiles its templates from static
// JSX (compile-time constants); the notes editor assigns `sanitizeNoteHtml(...)`
// (modules/notes/Editor.tsx); the key card injects module-generated QR SVG
// (modules/keys); the composer parses into a detached container it never renders
// (components/compose/richtext.ts); the plugin host builds its own iframe srcdoc
// (plugins-ui/host.ts). ProseMirror's own `ProseMirrorClipboard` policy reuses this
// default when present (prosemirror-view checks `trustedTypes.defaultPolicy` first).
//
// It deliberately exposes ONLY `createHTML`: the app has no dynamic script or
// script-URL sink (workers load via `new URL(...)`, which is not a TT sink, and
// `script-src 'self'` blocks script injection regardless), so those sinks stay
// fail-closed rather than being handed a passthrough. The DOM lib in our TS target
// does not yet declare `window.trustedTypes`; narrow it locally (no runtime dep).
const tt = (
  window as Window & {
    trustedTypes?: {
      readonly defaultPolicy: unknown;
      createPolicy(name: string, rules: { createHTML: (input: string) => string }): unknown;
    };
  }
).trustedTypes;
if (tt !== undefined && tt.defaultPolicy === null) {
  tt.createPolicy('default', { createHTML: (html) => html });
}

const root = document.getElementById('root');
if (root === null) {
  throw new Error('#root element not found');
}

// LocaleProvider (i18n foundation, plan §6 e0): negotiates the active locale,
// loads the critical `en` catalog, drives `<html lang/dir>` + reduced-motion.
// Wraps the whole tree so `t()` is reactive everywhere.
render(
  () => (
    <LocaleProvider>
      <App />
    </LocaleProvider>
  ),
  root,
);
