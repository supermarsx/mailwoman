// The paging half of the windowing maths (t22-e5b).
//
// A separate file from `virtual.test.ts` so that file stays the control for the
// pre-paging behaviour of `computeWindow`/`sameWindow`, untouched.
//
// These two functions are where the units change: `computeWindow` works in row
// slots, `projectedRowCount` is handed a VISUAL row count and a MESSAGE count and
// has to reconcile them, and `splitWindow` decides which slots may claim to be
// messages. An off-by-one in either is invisible in a screenshot.

import { describe, it, expect } from 'vitest';
import { computeWindow, projectedRowCount, splitWindow } from './virtual.ts';

describe('projectedRowCount', () => {
  it('spans the whole query when only one page is loaded', () => {
    // The headline case: 50 loaded of a 20 000-message folder.
    expect(projectedRowCount(50, 20_000, 50)).toBe(20_000);
  });

  it('shrinks the pending tail as pages arrive, not the span', () => {
    expect(projectedRowCount(100, 20_000, 100)).toBe(20_000);
    expect(projectedRowCount(150, 20_000, 150)).toBe(20_000);
  });

  it('returns only what is loaded when the server reported no total', () => {
    // Unknown must never be guessed at: this is the pre-paging behaviour, and it
    // is what every component spec written before paging still exercises.
    expect(projectedRowCount(200, null, 200)).toBe(200);
    expect(projectedRowCount(0, null, 0)).toBe(0);
  });

  it('counts collapsed conversations as the rows they render, not the messages they hold', () => {
    // 100 messages loaded, folded into 40 visual rows, out of 20 000. The loaded
    // region is 40 rows; the tail is the 19 900 messages not yet fetched.
    expect(projectedRowCount(40, 20_000, 100)).toBe(19_940);
  });

  it('never returns a negative tail when the loaded extent overruns the total', () => {
    // Happens transiently: `total` is from the query that has already been
    // superseded by a longer one, or rows arrived from a push before a recount.
    expect(projectedRowCount(60, 50, 60)).toBe(60);
    expect(projectedRowCount(50, 0, 50)).toBe(50);
  });

  it('is exhausted-safe: a fully loaded folder spans exactly its rows', () => {
    expect(projectedRowCount(120, 120, 120)).toBe(120);
  });
});

describe('splitWindow', () => {
  const win = (startIndex: number, endIndex: number) => ({
    startIndex,
    endIndex,
    offsetY: startIndex * 72,
    totalHeight: 0,
  });

  it('is all loaded when the window sits inside the loaded rows', () => {
    expect(splitWindow(win(0, 15), 50)).toEqual({
      loadedStart: 0,
      loadedEnd: 15,
      pendingStart: 50,
      pendingEnd: 50, // empty
    });
  });

  it('splits a window that straddles the loaded boundary', () => {
    expect(splitWindow(win(40, 70), 50)).toEqual({
      loadedStart: 40,
      loadedEnd: 50,
      pendingStart: 50,
      pendingEnd: 70,
    });
  });

  it('is all pending when the window is entirely past what is loaded', () => {
    expect(splitWindow(win(5_000, 5_021), 50)).toEqual({
      loadedStart: 50,
      loadedEnd: 50, // empty
      pendingStart: 5_000,
      pendingEnd: 5_021,
    });
  });

  it('never lets the two halves overlap, at any boundary', () => {
    // The failure this guards is a placeholder rendered ON TOP of a real row at
    // the same offset — a flicker, not an obvious bug. Swept rather than spot-checked.
    for (let loaded = 0; loaded <= 12; loaded += 1) {
      for (let start = 0; start <= 12; start += 1) {
        for (let end = start; end <= 12; end += 1) {
          const s = splitWindow(win(start, end), loaded);
          expect(s.loadedEnd).toBeLessThanOrEqual(s.pendingStart);
          // Every slot in the window is claimed exactly once, by one half or neither.
          const covered = Math.max(0, s.loadedEnd - s.loadedStart) + Math.max(0, s.pendingEnd - s.pendingStart);
          expect(covered).toBe(end - start);
        }
      }
    }
  });

  it('treats a negative loaded count as nothing loaded', () => {
    expect(splitWindow(win(0, 10), -5)).toEqual({
      loadedStart: 0,
      loadedEnd: 0,
      pendingStart: 0,
      pendingEnd: 10,
    });
  });
});

describe('computeWindow over a projected span', () => {
  it('gives the folder a scrollbar, not the page', () => {
    // 50 loaded of 20 000 at 72px: the spacer is the folder's height, which is
    // what made the pre-paging list show a 3 600px scrollbar for 20 000 messages.
    const span = projectedRowCount(50, 20_000, 50);
    expect(computeWindow(0, 600, 72, span).totalHeight).toBe(20_000 * 72);
    expect(computeWindow(0, 600, 72, 50).totalHeight).toBe(3_600);
  });

  it('lets the window travel past the loaded rows, which is what makes slots pending', () => {
    const span = projectedRowCount(50, 20_000, 50);
    const w = computeWindow(5_000 * 72, 600, 72, span);
    expect(w.startIndex).toBeGreaterThan(50);
    expect(splitWindow(w, 50).loadedEnd - splitWindow(w, 50).loadedStart).toBe(0);
  });
});
