import { describe, expect, it } from 'vitest';
import { autoScrollSpeed, boxBetween, clampBox, extendSpan, intersects, modeFor, passedThreshold, resolveSelection } from './marqueeGeometry.ts';

describe('marquee geometry', () => {
  it('orders corners whichever way the drag went', () => {
    expect(boxBetween({ x: 10, y: 40 }, { x: 2, y: 5 })).toEqual({ left: 2, top: 5, right: 10, bottom: 40 });
  });
  it('intersects on overlap or a shared edge, never for empty boxes', () => {
    const row = { left: 0, top: 20, right: 100, bottom: 40 };
    expect(intersects({ left: 10, top: 30, right: 20, bottom: 60 }, row)).toBe(true);
    expect(intersects({ left: 10, top: 40, right: 20, bottom: 60 }, row)).toBe(true);
    expect(intersects({ left: 10, top: 41, right: 20, bottom: 60 }, row)).toBe(false);
    expect(intersects({ left: 200, top: 0, right: 220, bottom: 100 }, row)).toBe(false);
    expect(intersects({ left: 10, top: 30, right: 20, bottom: 60 }, { left: 0, top: 0, right: 0, bottom: 0 })).toBe(false);
  });
  it('clamps to bounds', () => {
    const bounds = { left: 0, top: 0, right: 100, bottom: 100 };
    expect(clampBox({ left: -10, top: 50, right: 40, bottom: 200 }, bounds)).toEqual({ left: 0, top: 50, right: 40, bottom: 100 });
    expect(clampBox({ left: 200, top: 0, right: 300, bottom: 10 }, bounds)).toBeNull();
  });
  it('needs a minimum travel to count as a drag', () => {
    expect(passedThreshold({ x: 0, y: 0 }, { x: 2, y: 2 }, 4)).toBe(false);
    expect(passedThreshold({ x: 0, y: 0 }, { x: 3, y: 3 }, 4)).toBe(true);
  });
  it('auto-scroll speed: still in the middle, faster nearer and beyond the edge, signed by direction', () => {
    expect(autoScrollSpeed(300, 0, 600)).toBe(0);
    const near = autoScrollSpeed(580, 0, 600), nearer = autoScrollSpeed(598, 0, 600), past = autoScrollSpeed(640, 0, 600);
    expect(near).toBeGreaterThan(0);
    expect(nearer).toBeGreaterThan(near);
    expect(past).toBeGreaterThan(nearer);
    expect(past).toBeLessThanOrEqual(36);
    expect(autoScrollSpeed(10, 0, 600)).toBeLessThan(0);
    expect(autoScrollSpeed(-40, 0, 600)).toBe(-autoScrollSpeed(640, 0, 600));
  });
  it('shrinks the scroll zone for a tiny viewport and never scrolls a zero-height one', () => {
    expect(autoScrollSpeed(5, 0, 0)).toBe(0);
    expect(autoScrollSpeed(10, 0, 20)).toBe(0);
  });
  it('maps modifiers to modes (shift wins)', () => {
    expect(modeFor({ ctrlKey: false, metaKey: false, shiftKey: false })).toBe('replace');
    expect(modeFor({ ctrlKey: true, metaKey: false, shiftKey: false })).toBe('add');
    expect(modeFor({ ctrlKey: false, metaKey: true, shiftKey: false })).toBe('add');
    expect(modeFor({ ctrlKey: true, metaKey: false, shiftKey: true })).toBe('extend');
  });
  it('resolves replace, add and extend', () => {
    const order = ['a', 'b', 'c', 'd', 'e'];
    expect([...resolveSelection('replace', order, new Set(['a']), new Set(['c']))]).toEqual(['c']);
    expect([...resolveSelection('add', order, new Set(['a']), new Set(['c']))].sort()).toEqual(['a', 'c']);
    expect([...resolveSelection('extend', order, new Set(['a']), new Set(['c']))]).toEqual(['a', 'b', 'c']);
  });
  it('extend keeps selected ids outside the visible order and handles an empty base', () => {
    expect([...extendSpan(['a', 'b', 'c'], new Set(['gone']), new Set(['b']))].sort()).toEqual(['b', 'gone']);
    expect([...extendSpan(['a', 'b', 'c'], new Set(), new Set())]).toEqual([]);
  });
});
