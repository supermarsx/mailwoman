import { describe, expect, it } from 'vitest';
import { placePopover } from './placement.ts';

const view = { left: 0, top: 0, width: 1440, height: 900 };
const size = { width: 296, height: 420 };
const anchor = (left: number, top: number, width = 32, height = 32) => ({ left, top, width, height });

describe('placePopover (t81)', () => {
  it('sits below the trigger, start-aligned, when it fits', () => {
    expect(placePopover(anchor(100, 100), size, view)).toEqual({ left: 100, top: 136, side: 'below', maxHeight: 884 });
  });
  it('flips above when the bottom has no room and the top has', () => {
    const p = placePopover(anchor(100, 800), size, view);
    expect(p.side).toBe('above');
    expect(p.top).toBe(800 - 4 - 420);
  });
  it('stays below and scrolls in the room there when neither side fits but below has more', () => {
    const small = { left: 0, top: 0, width: 390, height: 500 };
    const p = placePopover(anchor(10, 100), size, small);
    expect(p).toMatchObject({ side: 'below', top: 136, maxHeight: 500 - 8 - 136 });
  });
  it('takes the side with more room when neither fits', () => {
    const small = { left: 0, top: 0, width: 390, height: 500 };
    const p = placePopover(anchor(10, 400), size, small);
    expect(p).toMatchObject({ side: 'above', top: 8, maxHeight: 400 - 4 - 8 });
  });
  it('lays over the trigger, inside the viewport, when no side has usable room', () => {
    const short = { left: 0, top: 0, width: 390, height: 300 };
    const p = placePopover(anchor(10, 130), size, short);
    expect(p.maxHeight).toBe(284);
    expect(p.top).toBe(8);
  });
  it('shifts left at the right edge and right at the left edge', () => {
    expect(placePopover(anchor(1420, 100, 16), size, view).left).toBe(1440 - 8 - 296);
    expect(placePopover(anchor(-20, 100), size, view).left).toBe(8);
  });
  it('lines up with the end edge in RTL', () => {
    expect(placePopover(anchor(600, 100, 40), size, view, { rtl: true }).left).toBe(640 - 296);
  });
  it('pins to the margin when wider than the viewport, and scrolls in the room below', () => {
    const tiny = { left: 0, top: 0, width: 200, height: 300 };
    const p = placePopover(anchor(50, 50), size, tiny);
    expect(p).toMatchObject({ left: 8, top: 86, side: 'below', maxHeight: 206 });
  });
  it('honours a scrolled visual viewport (pinch zoom)', () => {
    const vv = { left: 200, top: 300, width: 390, height: 600 };
    const p = placePopover(anchor(560, 320), size, vv);
    expect(p.left).toBe(200 + 390 - 8 - 296);
    expect(p.top).toBe(356);
  });
});
