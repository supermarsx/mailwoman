import { describe, expect, it } from 'vitest';
import { plainLanguageOf } from './model.ts';
import { setActiveLocale } from '../../i18n/registry.ts';

const base = 'This message passed DKIM sender authentication.';

describe('plainLanguageOf', () => {
  it('builds the anomaly sentence from the count with a plural', () => {
    expect(plainLanguageOf({ plainLanguage: `${base} 1 header anomaly/anomalies detected.`, anomalies: ['dateSkew'] }))
      .toBe(`${base} 1 header anomaly detected.`);
    expect(plainLanguageOf({ plainLanguage: `${base} 2 header anomaly/anomalies detected.`, anomalies: ['dateSkew', 'punycodeSender'] }))
      .toBe(`${base} 2 header anomalies detected.`);
  });
  it('keeps the server text when there is nothing to replace', () => {
    expect(plainLanguageOf({ plainLanguage: base, anomalies: [] })).toBe(base);
    expect(plainLanguageOf({ plainLanguage: base, anomalies: ['dateSkew'] })).toBe(base);
  });
  it('uses the active locale', () => {
    setActiveLocale('en');
    expect(plainLanguageOf({ plainLanguage: 'x 3 header anomaly/anomalies detected.', anomalies: [1, 2, 3] })).toBe('x 3 header anomalies detected.');
  });
});
