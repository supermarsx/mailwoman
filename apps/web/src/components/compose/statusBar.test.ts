import { describe, expect, it } from 'vitest';
import { inlineCapacity, orderItems, savedTime, splitItems, type StatusItem } from './statusBar.ts';

const item = (id: string, severity: StatusItem['severity']): StatusItem => ({ id, severity, icon: 'info', label: id });

describe('status bar model', () => {
  it('orders by severity and keeps the given order inside one severity', () => {
    const out = orderItems([item('saved', 'success'), item('b', 'warning'), item('a', 'danger'), item('c', 'warning'), item('n', 'neutral'), item('busy', 'busy')]);
    expect(out.map(i => i.id)).toEqual(['a', 'b', 'c', 'busy', 'n', 'saved']);
  });
  it('splits what fits from the overflow, most urgent inline', () => {
    const { inline, overflow } = splitItems([item('saved', 'success'), item('err', 'danger'), item('w', 'warning')], 2);
    expect(inline.map(i => i.id)).toEqual(['err', 'w']);
    expect(overflow.map(i => i.id)).toEqual(['saved']);
    expect(splitItems([item('a', 'neutral')], 3).overflow).toEqual([]);
  });
  it('derives a capacity from the width, never below one, with a roomy default when unmeasured', () => {
    expect(inlineCapacity(0)).toBe(6);
    expect(inlineCapacity(100)).toBe(1);
    expect(inlineCapacity(900)).toBe(3);
  });
  it('formats the saved time as hours and minutes', () => {
    expect(savedTime(new Date(2026, 0, 1, 14, 32), 'en-GB')).toBe('14:32');
  });
});
