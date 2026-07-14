import { render } from 'solid-js/web';
import { App } from './App.tsx';
import { LocaleProvider } from './i18n/index.ts';
import './styles/app.css';

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
