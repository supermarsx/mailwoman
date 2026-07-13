import { describe, it, expect } from 'vitest';
import { encodeQr, qrToSvg } from './qr.ts';

const FPR = 'ABCD1234ABCD1234ABCD1234ABCD1234ABCD1234';

/** A finder pattern occupies a 7×7 block: solid outer ring + 3×3 solid core. */
function assertFinder(m: boolean[][], top: number, left: number): void {
  for (let i = 0; i < 7; i++) {
    expect(m[top]![left + i]).toBe(true); // top edge
    expect(m[top + 6]![left + i]).toBe(true); // bottom edge
    expect(m[top + i]![left]).toBe(true); // left edge
    expect(m[top + i]![left + 6]).toBe(true); // right edge
  }
  expect(m[top + 1]![left + 1]).toBe(false); // inner white ring
  expect(m[top + 3]![left + 3]).toBe(true); // solid core
}

describe('encodeQr', () => {
  it('sizes the matrix to the chosen version (40 bytes → v3, 29 modules)', () => {
    const m = encodeQr(FPR);
    expect(m.length).toBe(29);
    expect(m[0]!.length).toBe(29);
  });

  it('places the three finder patterns at the corners', () => {
    const m = encodeQr(FPR);
    const n = m.length;
    assertFinder(m, 0, 0);
    assertFinder(m, 0, n - 7);
    assertFinder(m, n - 7, 0);
  });

  it('lays a timing pattern along row/col 6 and sets the dark module', () => {
    const m = encodeQr(FPR);
    expect(m[6]![8]).toBe(true); // even column → dark
    expect(m[6]![9]).toBe(false); // odd column → light
    expect(m[m.length - 8]![8]).toBe(true); // mandatory dark module
  });

  it('is deterministic for the same payload', () => {
    expect(encodeQr(FPR)).toEqual(encodeQr(FPR));
  });

  it('grows the version for a longer payload', () => {
    const big = encodeQr('OPENPGP4FPR:' + FPR.repeat(3));
    expect(big.length).toBeGreaterThan(29);
  });

  it('throws rather than truncating an over-large payload', () => {
    expect(() => encodeQr('x'.repeat(2000))).toThrow(/too large/);
  });
});

describe('qrToSvg', () => {
  it('renders a self-contained SVG sized to the module grid + quiet zone', () => {
    const svg = qrToSvg(encodeQr(FPR), 4, 4);
    expect(svg).toMatch(/^<svg xmlns="http:\/\/www\.w3\.org\/2000\/svg"/);
    // (29 modules + 2×4 quiet) × 4px = 148
    expect(svg).toContain('width="148"');
    expect(svg).toContain('<rect');
  });
});
