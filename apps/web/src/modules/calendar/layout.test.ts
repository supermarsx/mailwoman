import { describe, expect, it } from 'vitest';
import { inAllDayRow, inkOn, layoutColumns } from './layout.ts';

const iv = (start: number, end: number): { start: number; end: number } => ({ start, end });

describe('layoutColumns', () => {
  it('gives non-overlapping events the full width', () => {
    expect(layoutColumns([iv(540, 600), iv(600, 660)])).toEqual([
      { col: 0, cols: 1 },
      { col: 0, cols: 1 },
    ]);
  });

  it('puts concurrent events side by side', () => {
    expect(layoutColumns([iv(540, 600), iv(550, 620), iv(560, 580)])).toEqual([
      { col: 0, cols: 3 },
      { col: 1, cols: 3 },
      { col: 2, cols: 3 },
    ]);
  });

  it('reuses a freed column and shares the cluster column count', () => {
    // A 9-11, B 9-10, C 10-11: C fits under B, so two columns.
    expect(layoutColumns([iv(540, 660), iv(540, 600), iv(600, 660)])).toEqual([
      { col: 0, cols: 2 },
      { col: 1, cols: 2 },
      { col: 1, cols: 2 },
    ]);
  });

  it('keeps separate clusters independent and returns slots in input order', () => {
    const slots = layoutColumns([iv(900, 960), iv(540, 600), iv(570, 630)]);
    expect(slots).toEqual([
      { col: 0, cols: 1 },
      { col: 0, cols: 2 },
      { col: 1, cols: 2 },
    ]);
  });

  it('handles an empty list', () => {
    expect(layoutColumns([])).toEqual([]);
  });
});

describe('inAllDayRow', () => {
  const at = (h: number): Date => new Date(2026, 9, 29, h);
  it('takes all-day events and timed events of a day or more', () => {
    expect(inAllDayRow({ allDay: true, start: at(0), end: at(24) })).toBe(true);
    expect(inAllDayRow({ allDay: false, start: at(9), end: new Date(2026, 9, 31, 9) })).toBe(true);
  });
  it('leaves shorter timed events (even across midnight) in the grid', () => {
    expect(inAllDayRow({ allDay: false, start: at(9), end: at(10) })).toBe(false);
    expect(inAllDayRow({ allDay: false, start: at(22), end: at(26) })).toBe(false);
  });
});

describe('inkOn', () => {
  it('uses dark ink on light colours and light ink on dark ones', () => {
    expect(inkOn('#ffeb3b')).toBe('#111');
    expect(inkOn('#fff')).toBe('#111');
    expect(inkOn('#1e3a8a')).toBe('#fff');
    expect(inkOn('#000000')).toBe('#fff');
  });
  it('falls back to light ink for colours it cannot parse', () => {
    expect(inkOn('rebeccapurple')).toBe('#fff');
  });
});
