import { describe, it, expect } from 'vitest';
import { computeWindow } from './virtual.ts';

describe('computeWindow', () => {
  const ROW = 72;
  const VIEW = 600; // ~8.3 rows visible

  it('mounts only a small window of a 100k-row list', () => {
    const w = computeWindow(0, VIEW, ROW, 100_000);
    // 100k rows would be catastrophic to mount; the window is a tiny slice.
    expect(w.endIndex - w.startIndex).toBeLessThan(30);
    expect(w.totalHeight).toBe(100_000 * ROW);
  });

  it('advances the window as the user scrolls', () => {
    const top = computeWindow(0, VIEW, ROW, 100_000);
    expect(top.startIndex).toBe(0);

    const mid = computeWindow(50_000 * ROW, VIEW, ROW, 100_000);
    expect(mid.startIndex).toBeGreaterThan(49_000);
    expect(mid.offsetY).toBe(mid.startIndex * ROW);
    // The mounted slice brackets the scroll position.
    expect(mid.startIndex).toBeLessThanOrEqual(50_000);
    expect(mid.endIndex).toBeGreaterThan(50_000);
  });

  it('keeps overscan rows above the fold for smooth scrolling', () => {
    const w = computeWindow(100 * ROW, VIEW, ROW, 1000);
    expect(w.startIndex).toBeLessThan(100); // overscan mounts rows before the top
  });

  it('clamps at the end of the list', () => {
    const w = computeWindow(999_999 * ROW, VIEW, ROW, 1000);
    expect(w.endIndex).toBe(1000);
    expect(w.startIndex).toBeGreaterThanOrEqual(0);
  });

  it('handles an empty list', () => {
    const w = computeWindow(0, VIEW, ROW, 0);
    expect(w).toEqual({ startIndex: 0, endIndex: 0, offsetY: 0, totalHeight: 0 });
  });
});
