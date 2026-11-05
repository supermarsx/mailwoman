// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { attachMarquee } from './marqueeSelect.ts';

class TestPointerEvent extends MouseEvent {
  readonly pointerId: number; readonly pointerType: string;
  constructor(type: string, init: PointerEventInit = {}) { super(type, { bubbles: true, cancelable: true, ...init }); this.pointerId = init.pointerId ?? 1; this.pointerType = init.pointerType ?? 'mouse'; }
}
const ROW = 30;
let frames: FrameRequestCallback[] = [];
const flush = (): void => { const run = frames; frames = []; run.forEach(cb => cb(0)); };

function rect(left: number, top: number, width: number, height: number): DOMRect {
  return { left, top, width, height, right: left + width, bottom: top + height, x: left, y: top, toJSON: () => ({}) } as DOMRect;
}
interface World { scroller: HTMLElement; rows: { id: string; el: HTMLElement }[]; gap: HTMLElement; button: HTMLButtonElement; selection: Set<string>; changes: Set<string>[]; stop: () => void; clicks: number }

/** A 200x300 scroller at (0,0) with six 30px rows at content y = 0, 30, ...; hidden rows have no box. */
function build(options: { hiddenIds?: string[] } = {}): World {
  const scroller = document.createElement('div');
  const rows = ['a', 'b', 'c', 'd', 'e', 'f'].map((id, index) => {
    const el = document.createElement('div'); el.dataset.item = id; el.textContent = id; scroller.appendChild(el);
    el.getBoundingClientRect = () => options.hiddenIds?.includes(id) ? rect(0, 0, 0, 0) : rect(0, index * ROW - scroller.scrollTop, 200, ROW - 2);
    return { id, el };
  });
  const gap = document.createElement('div'); scroller.appendChild(gap);
  const button = document.createElement('button'); scroller.appendChild(button);
  document.body.appendChild(scroller);
  scroller.getBoundingClientRect = () => rect(0, 0, 200, 300);
  Object.defineProperty(scroller, 'clientWidth', { value: 185, configurable: true });
  Object.defineProperty(scroller, 'clientHeight', { value: 300, configurable: true });
  // jsdom clamps nothing: emulate a 1000px tall content so scrollTop stays in [0, 700].
  let top = 0;
  Object.defineProperty(scroller, 'scrollTop', { get: () => top, set: (value: number) => { top = Math.max(0, Math.min(700, value)); }, configurable: true });
  const world: World = { scroller, rows, gap, button, selection: new Set(), changes: [], stop: () => {}, clicks: 0 };
  world.stop = attachMarquee({
    container: scroller, items: () => rows, selection: () => world.selection,
    onChange: next => { world.selection = new Set(next); world.changes.push(new Set(next)); },
    onEmptyClick: () => { world.clicks++; world.selection = new Set(); },
  });
  return world;
}
const fire = (target: EventTarget, type: string, init: PointerEventInit = {}): void => { target.dispatchEvent(new TestPointerEvent(type, init)); };
const ids = (world: World): string[] => [...world.selection].sort();

beforeEach(() => {
  vi.stubGlobal('PointerEvent', TestPointerEvent);
  vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => { frames.push(cb); return frames.length; });
  vi.stubGlobal('cancelAnimationFrame', () => { frames = []; });
});
afterEach(() => { document.body.innerHTML = ''; frames = []; vi.unstubAllGlobals(); });

describe('attachMarquee', () => {
  it('a plain drag from empty space replaces the selection with the rows it touches, live', () => {
    const w = build(); w.selection = new Set(['f']);
    fire(w.gap, 'pointerdown', { clientX: 195, clientY: 200 });
    fire(document, 'pointermove', { clientX: 190, clientY: 70 }); flush();
    expect(document.querySelector('[data-marquee-band]')).not.toBeNull();
    expect(ids(w)).toEqual(['c', 'd', 'e', 'f']);
    fire(document, 'pointermove', { clientX: 190, clientY: 100 }); flush();
    expect(ids(w)).toEqual(['d', 'e', 'f']);
    fire(document, 'pointerup', { clientX: 190, clientY: 100 });
    expect(ids(w)).toEqual(['d', 'e', 'f']);
    expect(document.querySelector('[data-marquee-band]')).toBeNull();
  });
  it('Ctrl or Cmd adds to the existing selection', () => {
    for (const key of ['ctrlKey', 'metaKey'] as const) {
      const w = build(); w.selection = new Set(['a']);
      fire(w.gap, 'pointerdown', { clientX: 195, clientY: 200, [key]: true });
      fire(document, 'pointermove', { clientX: 190, clientY: 100, [key]: true }); flush();
      expect(ids(w)).toEqual(['a', 'd', 'e', 'f']);
      fire(document, 'pointerup'); w.stop(); document.body.innerHTML = '';
    }
  });
  it('Shift extends the selection to the covering range', () => {
    const w = build(); w.selection = new Set(['a']);
    fire(w.gap, 'pointerdown', { clientX: 195, clientY: 200, shiftKey: true });
    fire(document, 'pointermove', { clientX: 190, clientY: 100, shiftKey: true }); flush();
    expect(ids(w)).toEqual(['a', 'b', 'c', 'd', 'e', 'f']);
  });
  it('rows with no box (hidden, collapsed) are never selected', () => {
    const w = build({ hiddenIds: ['c'] });
    fire(w.gap, 'pointerdown', { clientX: 195, clientY: 200 });
    fire(document, 'pointermove', { clientX: 190, clientY: 10 }); flush();
    expect(ids(w)).toEqual(['a', 'b', 'd', 'e', 'f']);
  });
  it('Escape cancels the drag and restores the selection from before it', () => {
    const w = build(); w.selection = new Set(['f']);
    fire(w.gap, 'pointerdown', { clientX: 195, clientY: 200 });
    fire(document, 'pointermove', { clientX: 190, clientY: 20 }); flush();
    expect(ids(w)).not.toEqual(['f']);
    document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }));
    expect(ids(w)).toEqual(['f']);
    expect(document.querySelector('[data-marquee-band]')).toBeNull();
    fire(document, 'pointermove', { clientX: 190, clientY: 20 }); flush();
    expect(ids(w)).toEqual(['f']);
  });
  it('a click on empty space without dragging clears; a tiny wobble is still a click', () => {
    const w = build(); w.selection = new Set(['a', 'b']);
    fire(w.gap, 'pointerdown', { clientX: 195, clientY: 200 });
    fire(document, 'pointermove', { clientX: 196, clientY: 201 }); flush();
    fire(document, 'pointerup', { clientX: 196, clientY: 201 });
    expect(w.clicks).toBe(1); expect(ids(w)).toEqual([]);
    expect(document.querySelector('[data-marquee-band]')).toBeNull();
  });
  it('a modified click on empty space keeps the selection', () => {
    const w = build(); w.selection = new Set(['a']);
    fire(w.gap, 'pointerdown', { clientX: 195, clientY: 200, ctrlKey: true });
    fire(document, 'pointerup');
    expect(w.clicks).toBe(0); expect(ids(w)).toEqual(['a']);
  });
  it('does not start from a row, a control, the scrollbar, a touch or a secondary button', () => {
    const w = build();
    fire(w.rows[0]!.el, 'pointerdown', { clientX: 10, clientY: 10 });
    fire(w.button, 'pointerdown', { clientX: 10, clientY: 250 });
    fire(w.scroller, 'pointerdown', { clientX: 195, clientY: 100 }); // scrollbar gutter (x beyond clientWidth)
    fire(w.gap, 'pointerdown', { clientX: 10, clientY: 250, pointerType: 'touch' });
    fire(w.gap, 'pointerdown', { clientX: 10, clientY: 250, button: 2 });
    fire(document, 'pointermove', { clientX: 190, clientY: 20 }); flush();
    fire(document, 'pointerup');
    expect(w.changes).toEqual([]); expect(w.clicks).toBe(0);
    expect(document.querySelector('[data-marquee-band]')).toBeNull();
  });
  it('starts from the scroller background inside its content box', () => {
    const w = build();
    fire(w.scroller, 'pointerdown', { clientX: 100, clientY: 250 });
    fire(document, 'pointermove', { clientX: 100, clientY: 70 }); flush();
    expect(ids(w)).toEqual(['c', 'd', 'e', 'f']);
  });
  it('auto-scrolls near the bottom edge and keeps selecting rows as they scroll into view', () => {
    const w = build();
    fire(w.gap, 'pointerdown', { clientX: 195, clientY: 100 });
    fire(document, 'pointermove', { clientX: 190, clientY: 296 }); flush();
    expect(w.scroller.scrollTop).toBeGreaterThan(0);
    expect(frames.length).toBeGreaterThan(0);
    const after = w.scroller.scrollTop;
    flush();
    expect(w.scroller.scrollTop).toBeGreaterThan(after);
    expect(ids(w)).toContain('f');
  });
  it('auto-scrolls up near the top edge', () => {
    const w = build(); w.scroller.scrollTop = 100;
    fire(w.gap, 'pointerdown', { clientX: 195, clientY: 100 });
    fire(document, 'pointermove', { clientX: 190, clientY: 3 }); flush();
    expect(w.scroller.scrollTop).toBeLessThan(100);
  });
  it('coalesces pointer moves to one update per animation frame', () => {
    const w = build();
    fire(w.gap, 'pointerdown', { clientX: 195, clientY: 200 });
    for (let y = 190; y > 100; y -= 10) fire(document, 'pointermove', { clientX: 190, clientY: y });
    expect(frames).toHaveLength(1);
    flush();
    expect(w.changes).toHaveLength(1);
  });
  it('detach removes the listeners', () => {
    const w = build(); w.stop();
    fire(w.gap, 'pointerdown', { clientX: 195, clientY: 200 });
    fire(document, 'pointermove', { clientX: 190, clientY: 20 }); flush();
    expect(w.changes).toEqual([]);
  });
});
