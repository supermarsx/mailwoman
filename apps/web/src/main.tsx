import { render } from 'solid-js/web';
import { App } from './App.tsx';
import { LocaleProvider } from './i18n/index.ts';
import { installTrustedTypesPolicy } from './security/trustedTypes.ts';
import { bootLocale } from './screens/Settings/language.ts';
import './styles/app.css';
// After app.css: one size and accent for every checkbox and radio (t32-h12).
import './styles/checks.css.ts';

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
// Wraps the whole tree so `t()` is reactive everywhere. The language chosen in
// Settings (`ui.language`, kept in localStorage `mw.locale`) wins over the
// browser's languages from the first paint, before sign-in.
const startLocale = bootLocale();
render(
  () => (
    <LocaleProvider {...(startLocale !== null ? { locale: startLocale } : {})}>
      <App />
    </LocaleProvider>
  ),
  root,
);
