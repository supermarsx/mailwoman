import '@testing-library/jest-dom/vitest';
import { afterEach } from 'vitest';
import { setupI18nForTests } from './i18n.ts';
import { resetToasts } from '../components/toast/toast.ts';
import { forgetSession } from '../state/sessionRestore.ts';
import { WORKSPACE_STORAGE_KEY } from '../realtime/workspaceStorage.ts';

// Seed the i18n registry with the `en` catalogs so `t(id)` resolves to English
// synchronously — keeps literal-text assertions green as strings get wrapped.
setupI18nForTests();

// The toast store (components/toast) is module state: a toast one test raised
// must not be on screen in the next.
afterEach(() => resetToasts());

// t109: what a browser tab keeps to restore after a reload (sessionStorage and
// the address-bar target) is per tab; each test is a fresh tab.
afterEach(() => {
  forgetSession();
  try {
    for (const key of Object.keys(sessionStorage)) if (key.startsWith(`${WORKSPACE_STORAGE_KEY}:`)) sessionStorage.removeItem(key);
  } catch { /* no storage */ }
});
