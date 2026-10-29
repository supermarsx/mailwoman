import { afterEach, describe, expect, it } from 'vitest';
import { setActiveLocale } from './registry.ts';
import { durationFormatterFor, durationUnit, formatDurationMicros, formatDurationMs } from './format.ts';

// U+03BC GREEK SMALL LETTER MU, the symbol CLDR uses for "microsecond".
const MU = 'μ';

afterEach(() => setActiveLocale('en'));

describe('formatDurationMs', () => {
  it('scales the unit: ns, μs, ms, s, min, h', () => {
    expect(formatDurationMs(0.00042)).toBe('420 ns');
    expect(formatDurationMs(0.42)).toBe(`420 ${MU}s`);
    expect(formatDurationMs(1.25)).toBe('1.25 ms');
    expect(formatDurationMs(37)).toBe('37 ms');
    expect(formatDurationMs(2400)).toBe('2.4 s');
    expect(formatDurationMs(150_000)).toBe('2.5 min');
    expect(formatDurationMs(5_400_000)).toBe('1.5 h');
    expect(formatDurationMs(3_600_000 * 1234)).toBe('1,234 h');
  });

  it('switches unit at 999 μs / 1 ms, including after rounding', () => {
    expect(formatDurationMs(0.999)).toBe(`999 ${MU}s`);
    expect(formatDurationMs(0.9996)).toBe('1 ms');
    expect(formatDurationMs(1)).toBe('1 ms');
    expect(formatDurationMs(999.4)).toBe('999 ms');
    expect(formatDurationMs(999.6)).toBe('1 s');
    expect(formatDurationMs(59_999)).toBe('1 min');
    expect(formatDurationMs(0.000999)).toBe('999 ns');
    expect(formatDurationMs(0.0009996)).toBe(`1 ${MU}s`);
  });

  it('keeps three significant digits', () => {
    expect(formatDurationMs(1.2345)).toBe('1.23 ms');
    expect(formatDurationMs(12.345)).toBe('12.3 ms');
    expect(formatDurationMs(123.45)).toBe('123 ms');
    expect(formatDurationMs(0.3)).toBe(`300 ${MU}s`);
    expect(formatDurationMs(0.0123)).toBe(`12.3 ${MU}s`);
    expect(formatDurationMs(1.25, { significantDigits: 2 })).toBe('1.3 ms');
  });

  it('never shows a measured non-zero value as zero', () => {
    for (const v of [0.4, 0.04, 0.004, 0.0004, 0.00004, 1e-9]) {
      expect(formatDurationMs(v)).not.toMatch(/^0(\.0*)? /);
    }
    expect(formatDurationMs(1e-7)).toBe('0.1 ns');
  });

  it('shows "<step" below the source resolution, zero included', () => {
    expect(formatDurationMs(0, { resolutionMs: 1 })).toBe('<1 ms');
    expect(formatDurationMs(0.0004, { resolutionMs: 0.001 })).toBe(`<1 ${MU}s`);
    expect(formatDurationMs(0, { resolutionMs: 1000 })).toBe('<1 s');
    expect(formatDurationMs(3, { resolutionMs: 1 })).toBe('3 ms');
    expect(formatDurationMs(0.5, { resolutionMs: null })).toBe(`500 ${MU}s`);
  });

  it('guards zero, negative, NaN and infinity', () => {
    expect(formatDurationMs(0)).toBe('0 ms');
    expect(formatDurationMs(-0)).toBe('0 ms');
    expect(formatDurationMs(-2400)).toBe('-2.4 s');
    expect(formatDurationMs(-0.42)).toBe(`-420 ${MU}s`);
    expect(formatDurationMs(Number.NaN)).toBe('—');
    expect(formatDurationMs(Infinity)).toBe('—');
    expect(formatDurationMs(-Infinity)).toBe('—');
    expect(formatDurationMs(undefined as unknown as number)).toBe('—');
  });

  it('uses locale digits in pt-PT', () => {
    setActiveLocale('pt-PT');
    expect(formatDurationMs(1.25)).toBe('1,25 ms');
    expect(formatDurationMs(0.42)).toBe(`420 ${MU}s`);
    expect(formatDurationMs(2400)).toBe('2,4 s');
    expect(formatDurationMs(5_400_000)).toBe('1,5 h');
    expect(formatDurationMs(0, { resolutionMs: 0.001 })).toBe(`<1 ${MU}s`);
  });

  it('takes microseconds through formatDurationMicros', () => {
    expect(formatDurationMicros(420)).toBe(`420 ${MU}s`);
    expect(formatDurationMicros(1250)).toBe('1.25 ms');
    expect(formatDurationMicros(0)).toBe('0 ms');
  });
});

describe('durationUnit', () => {
  it('picks the unit from the rounded value', () => {
    expect(durationUnit(0)).toBe('millisecond');
    expect(durationUnit(0.5)).toBe('microsecond');
    expect(durationUnit(0.9996)).toBe('millisecond');
    expect(durationUnit(-2400)).toBe('second');
    expect(durationUnit(Number.NaN)).toBe('millisecond');
  });
});

describe('durationFormatterFor', () => {
  it('uses one unit for a whole set, chosen from the largest value', () => {
    const fmt = durationFormatterFor([0.3, 12, 380, null]);
    expect(fmt(0.3)).toBe('0.3 ms');
    expect(fmt(12)).toBe('12 ms');
    expect(fmt(380)).toBe('380 ms');
    const seconds = durationFormatterFor([0.3, 2400]);
    expect(seconds(0.3)).toBe('0.0003 s');
    expect(seconds(2400)).toBe('2.4 s');
    const micro = durationFormatterFor([0.001, 0.123, 0.999]);
    expect(micro(0.001)).toBe(`1 ${MU}s`);
    expect(micro(0.999)).toBe(`999 ${MU}s`);
  });

  it('keeps resolution and guards in a shared unit', () => {
    const fmt = durationFormatterFor([0, 50], { resolutionMs: 1 });
    expect(fmt(0)).toBe('<1 ms');
    expect(fmt(Number.NaN)).toBe('—');
    expect(durationFormatterFor([])(0)).toBe('0 ms');
  });
});
