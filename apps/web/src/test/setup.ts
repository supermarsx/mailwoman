import '@testing-library/jest-dom/vitest';
import { afterEach } from 'vitest';
import { setupI18nForTests } from './i18n.ts';
import { resetToasts } from '../components/toast/toast.ts';

// Seed the i18n registry with the `en` catalogs so `t(id)` resolves to English
// synchronously — keeps literal-text assertions green as strings get wrapped.
setupI18nForTests();

// The toast store (components/toast) is module state: a toast one test raised
// must not be on screen in the next.
afterEach(() => resetToasts());
