import '@testing-library/jest-dom/vitest';
import { setupI18nForTests } from './i18n.ts';

// Seed the i18n registry with the `en` catalogs so `t(id)` resolves to English
// synchronously — keeps literal-text assertions green as strings get wrapped.
setupI18nForTests();
