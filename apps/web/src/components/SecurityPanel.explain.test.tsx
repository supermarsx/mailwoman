import { readFileSync } from 'node:fs';
import { describe, it, expect } from 'vitest';
import { EXPLAIN_FIELDS, EXPLAIN_SEVERITY } from './SecurityPanel.tsx';
import type { AuthResult, SecurityAnomaly } from '../api/security-types.ts';

const ANOMALIES: SecurityAnomaly[] = [
  'replyToMismatch',
  'envelopeFromDivergence',
  'messageIdDomainAnomaly',
  'dateSkew',
  'punycodeSender',
];
const NON_PASS: AuthResult[] = ['fail', 'none', 'neutral', 'temperror', 'permerror'];
const CODES = [...ANOMALIES, ...NON_PASS.map((r) => `auth-${r}`)];

describe('finding explanations', () => {
  for (const lang of ['en', 'pt-PT']) {
    const ftl = readFileSync(`locales/${lang}/security.ftl`, 'utf8');
    it(`${lang}: every emitted code has every explain key`, () => {
      for (const code of CODES) {
        for (const f of EXPLAIN_FIELDS) {
          expect(ftl, `${code}-${f}`).toMatch(new RegExp(`^security-explain-${code}-${f} = \\S`, 'm'));
        }
        expect(EXPLAIN_SEVERITY[code], code).toBeDefined();
        expect(ftl).toMatch(new RegExp(`^security-explain-${code}-severity = \\{ security-severity-${EXPLAIN_SEVERITY[code]} \\}`, 'm'));
      }
    });
  }
  it('every anomaly in the frozen type has a headline', () => {
    const ftl = readFileSync('locales/en/security.ftl', 'utf8');
    for (const a of ANOMALIES) expect(ftl).toContain(`security-anomaly-${a} =`);
  });
});

describe('security dialog layout source', () => {
  const css = readFileSync('src/components/security/security-panel.css.ts', 'utf8');
  it('close button is at inline-end via logical properties, panel fills width', () => {
    expect(css).toMatch(/dialogClose=style\(\{[^}]*marginInlineStart:'auto'/);
    expect(css).not.toMatch(/maxWidth: '32rem'/);
    expect(css).toMatch(/minmax\(0, 1fr\)/);
    expect(css).not.toMatch(/\b(left|right|marginLeft|marginRight|paddingLeft|paddingRight)\s*:/);
  });
});
