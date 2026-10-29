import { describe, expect, it } from 'vitest';
import { contrastHint, flatten, hsvToRgb, normalizeHex, parseHex, rgbToHsv, toHex } from './colorMath.ts';

describe('hex parsing (t81)', () => {
  it('accepts #rgb and #rrggbb with or without # and in any case', () => {
    expect(parseHex('#abc')).toEqual({ r: 0xaa, g: 0xbb, b: 0xcc, a: 1 });
    expect(parseHex('  1F4E79 ')).toEqual({ r: 0x1f, g: 0x4e, b: 0x79, a: 1 });
    expect(normalizeHex('#ABC')).toBe('#aabbcc');
  });
  it('rejects bad input', () => {
    for (const bad of ['', '#', '#ab', '#abcd', '#abcde', '#gggggg', 'red', '#1234567', 'rgb(1,2,3)', '##abc']) expect(parseHex(bad)).toBeNull();
  });
  it('accepts alpha forms only when the target supports alpha', () => {
    expect(parseHex('#11223380')).toBeNull();
    expect(parseHex('#11223380', true)).toEqual({ r: 0x11, g: 0x22, b: 0x33, a: 0.5 });
    expect(parseHex('#1238', true)?.a).toBeCloseTo(0.53, 2);
    expect(toHex({ r: 17, g: 34, b: 51, a: 0.5 }, true)).toBe('#11223380');
    expect(toHex({ r: 17, g: 34, b: 51, a: 1 }, true)).toBe('#112233');
    expect(toHex({ r: 17, g: 34, b: 51, a: 0.5 })).toBe('#112233');
  });
});

describe('HSV round trip', () => {
  it('round-trips every 17th colour', () => {
    for (let r = 0; r <= 255; r += 51) for (let g = 0; g <= 255; g += 51) for (let b = 0; b <= 255; b += 51) {
      expect(hsvToRgb(rgbToHsv({ r, g, b, a: 1 }))).toEqual({ r, g, b, a: 1 });
    }
  });
  it('knows the primaries', () => {
    expect(rgbToHsv({ r: 255, g: 0, b: 0, a: 1 })).toEqual({ h: 0, s: 1, v: 1, a: 1 });
    expect(rgbToHsv({ r: 0, g: 0, b: 255, a: 1 }).h).toBe(240);
    expect(toHex(hsvToRgb({ h: 120, s: 1, v: 1, a: 1 }))).toBe('#00ff00');
    expect(toHex(hsvToRgb({ h: 360, s: 1, v: 1, a: 1 }))).toBe('#ff0000');
  });
});

describe('contrast hint', () => {
  it('judges text at 4.5:1 with a large-text band at 3:1', () => {
    expect(contrastHint('#000000', '#ffffff', 'text')).toEqual({ ratio: 21, level: 'aa' });
    expect(contrastHint('#767676', '#ffffff', 'text')?.level).toBe('aa');
    expect(contrastHint('#949494', '#ffffff', 'text')?.level).toBe('large');
    expect(contrastHint('#bbbbbb', '#ffffff', 'text')?.level).toBe('fail');
  });
  it('judges badges and boundaries at 3:1', () => {
    expect(contrastHint('#949494', '#ffffff', 'non-text')?.level).toBe('aa');
    expect(contrastHint('#bbbbbb', '#ffffff', 'non-text')?.level).toBe('fail');
  });
  it('is null when a side cannot be measured', () => {
    expect(contrastHint('nope', '#ffffff', 'text')).toBeNull();
  });
  it('flattens a translucent colour over its backdrop', () => {
    expect(flatten({ r: 0, g: 0, b: 0, a: 0.5 }, { r: 255, g: 255, b: 255, a: 1 })).toEqual({ r: 128, g: 128, b: 128, a: 1 });
  });
});
