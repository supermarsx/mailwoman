import { describe, expect, it } from 'vitest';
import { createRoot } from 'solid-js';
import { bindHorizontalWheel, horizontalWheel } from './horizontalWheel.ts';

/** A strip `clientWidth` wide holding `scrollWidth` of content (jsdom does no layout). */
function strip(clientWidth: number, scrollWidth: number, scrollLeft = 0, dir: 'ltr' | 'rtl' = 'ltr') {
  const element = document.createElement('div');
  element.style.direction = dir;
  let left = scrollLeft;
  Object.defineProperty(element, 'clientWidth', { configurable: true, get: () => clientWidth });
  Object.defineProperty(element, 'scrollWidth', { configurable: true, get: () => scrollWidth });
  Object.defineProperty(element, 'scrollLeft', { configurable: true, get: () => left, set: (value: number) => { left = value; } });
  document.body.append(element);
  return element;
}

function wheel(element: HTMLElement, init: WheelEventInit): WheelEvent {
  const event = new WheelEvent('wheel', { bubbles: true, cancelable: true, ...init });
  element.dispatchEvent(event);
  return event;
}

describe('bindHorizontalWheel', () => {
  it('turns a vertical wheel into sideways scrolling on an overflowing strip', () => {
    const element = strip(300, 900);
    bindHorizontalWheel(element);
    const down = wheel(element, { deltaY: 100 });
    expect(down.defaultPrevented).toBe(true);
    expect(element.scrollLeft).toBe(100);
    const up = wheel(element, { deltaY: -40 });
    expect(up.defaultPrevented).toBe(true);
    expect(element.scrollLeft).toBe(60);
  });

  it('leaves the page scrolling over a strip that fits', () => {
    const element = strip(600, 600);
    bindHorizontalWheel(element);
    const event = wheel(element, { deltaY: 100 });
    expect(event.defaultPrevented).toBe(false);
    expect(element.scrollLeft).toBe(0);
  });

  it('lets the page scroll once the strip is at the end it is moving toward', () => {
    const atEnd = strip(300, 900, 600);
    bindHorizontalWheel(atEnd);
    expect(wheel(atEnd, { deltaY: 100 }).defaultPrevented).toBe(false);
    expect(atEnd.scrollLeft).toBe(600);
    expect(wheel(atEnd, { deltaY: -100 }).defaultPrevented).toBe(true);
    expect(atEnd.scrollLeft).toBe(500);

    const atStart = strip(300, 900, 0);
    bindHorizontalWheel(atStart);
    expect(wheel(atStart, { deltaY: -100 }).defaultPrevented).toBe(false);
    expect(atStart.scrollLeft).toBe(0);
  });

  it('clamps a step that would pass the end', () => {
    const element = strip(300, 900, 550);
    bindHorizontalWheel(element);
    expect(wheel(element, { deltaY: 100 }).defaultPrevented).toBe(true);
    expect(element.scrollLeft).toBe(600);
  });

  it('leaves horizontal gestures, Shift+wheel and Ctrl+wheel to the browser', () => {
    const element = strip(300, 900, 100);
    bindHorizontalWheel(element);
    expect(wheel(element, { deltaX: 80 }).defaultPrevented).toBe(false);
    expect(wheel(element, { deltaX: 80, deltaY: 20 }).defaultPrevented).toBe(false);
    expect(wheel(element, { deltaY: 100, shiftKey: true }).defaultPrevented).toBe(false);
    expect(wheel(element, { deltaY: 100, ctrlKey: true }).defaultPrevented).toBe(false);
    expect(element.scrollLeft).toBe(100);
  });

  it('converts line and page wheel units to pixels', () => {
    const lines = strip(300, 900);
    bindHorizontalWheel(lines);
    wheel(lines, { deltaY: 3, deltaMode: WheelEvent.DOM_DELTA_LINE });
    expect(lines.scrollLeft).toBe(99);

    const pages = strip(300, 900);
    bindHorizontalWheel(pages);
    wheel(pages, { deltaY: 1, deltaMode: WheelEvent.DOM_DELTA_PAGE });
    expect(pages.scrollLeft).toBe(300);
  });

  it('scrolls toward the inline end of a right-to-left strip', () => {
    const element = strip(300, 900, 0, 'rtl');
    bindHorizontalWheel(element);
    expect(wheel(element, { deltaY: 100 }).defaultPrevented).toBe(true);
    expect(element.scrollLeft).toBe(-100);
    expect(wheel(element, { deltaY: -100 }).defaultPrevented).toBe(true);
    expect(element.scrollLeft).toBe(0);
    expect(wheel(element, { deltaY: -100 }).defaultPrevented).toBe(false);
  });

  it('removes the listener when unbound, and when the owning component is disposed', () => {
    const bound = strip(300, 900);
    bindHorizontalWheel(bound)();
    expect(wheel(bound, { deltaY: 100 }).defaultPrevented).toBe(false);

    const owned = strip(300, 900);
    const dispose = createRoot(dispose => { horizontalWheel(owned); return dispose; });
    expect(wheel(owned, { deltaY: 100 }).defaultPrevented).toBe(true);
    dispose();
    expect(wheel(owned, { deltaY: 100 }).defaultPrevented).toBe(false);
    expect(owned.scrollLeft).toBe(100);
  });
});
