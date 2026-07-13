import { describe, it, expect } from 'vitest';
import { fingerprintWords, groupFingerprint } from './proquint.ts';

describe('fingerprintWords (proquint safe words)', () => {
  it('returns no words for an empty fingerprint', () => {
    expect(fingerprintWords('')).toEqual([]);
  });

  it('maps the all-zero and all-one 16-bit groups to the boundary proquints', () => {
    expect(fingerprintWords('0000')).toEqual(['babab']);
    expect(fingerprintWords('ffff')).toEqual(['zuzuz']);
  });

  it('emits one 5-letter word per 16 bits (10 words for a 160-bit fingerprint)', () => {
    const words = fingerprintWords('ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234');
    expect(words).toHaveLength(10);
    for (const w of words) expect(w).toMatch(/^[bdfghjklmnprstvz][aiou][bdfghjklmnprstvz][aiou][bdfghjklmnprstvz]$/);
  });

  it('is deterministic and ignores separators/case', () => {
    const a = fingerprintWords('AB CD:12 34');
    const b = fingerprintWords('abcd1234');
    expect(a).toEqual(b);
  });

  it('zero-pads a trailing partial group so any length is stable', () => {
    expect(fingerprintWords('ab')).toEqual(fingerprintWords('ab00'));
  });
});

describe('groupFingerprint', () => {
  it('upper-cases and groups into 4-char blocks', () => {
    expect(groupFingerprint('abcd1234ef01')).toBe('ABCD 1234 EF01');
  });
});
