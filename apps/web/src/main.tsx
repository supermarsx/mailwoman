import { render } from 'solid-js/web';
import { App } from './App.tsx';
import { LocaleProvider } from './i18n/index.ts';
import { installTrustedTypesPolicy } from './security/trustedTypes.ts';
import './styles/app.css';

// Trusted Types default policy (SPEC §7.4, 26.17; script-URL half added in 26.20
// t24-e13). MUST run before `render()` — Solid's first `template().innerHTML` boot
// write goes through it — and before any worker is constructed. The policy and the
// reasoning for how narrow it is live in `security/trustedTypes.ts`.
installTrustedTypesPolicy();

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
