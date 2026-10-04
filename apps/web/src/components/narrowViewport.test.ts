import { afterEach, describe, expect, it, vi } from 'vitest';
import { createRoot } from 'solid-js';
import { createNarrowViewport, focusListRow, NARROW_QUERY } from './narrowViewport.ts';

/** A controllable `MediaQueryList` stand-in (jsdom has no `matchMedia`). */
function stubMatchMedia(initial: boolean) {
  const listeners = new Set<(e: MediaQueryListEvent) => void>();
  const mql = {
    matches: initial,
    addEventListener: (_: 'change', fn: (e: MediaQueryListEvent) => void) => listeners.add(fn),
    removeEventListener: (_: 'change', fn: (e: MediaQueryListEvent) => void) => listeners.delete(fn),
  };
  const matchMedia = vi.fn((_query: string) => mql);
  vi.stubGlobal('matchMedia', matchMedia);
  return {
    matchMedia,
    listeners,
    set(matches: boolean): void {
      mql.matches = matches;
      for (const fn of listeners) fn({ matches } as MediaQueryListEvent);
    },
  };
}

afterEach(() => {
  vi.unstubAllGlobals();
  document.body.replaceChildren();
});

describe('createNarrowViewport', () => {
  it('reads as wide, and stays so, where matchMedia does not exist', () => {
    expect(typeof globalThis.matchMedia).not.toBe('function');
    createRoot((dispose) => {
      expect(createNarrowViewport()()).toBe(false);
      dispose();
    });
  });

  it('asks about the app.css breakpoint and follows the media query live', () => {
    const media = stubMatchMedia(true);
    createRoot((dispose) => {
      const narrow = createNarrowViewport();
      expect(media.matchMedia).toHaveBeenCalledWith(NARROW_QUERY);
      expect(NARROW_QUERY).toBe('(max-width: 760px)');
      expect(narrow()).toBe(true);
      media.set(false);
      expect(narrow()).toBe(false);
      media.set(true);
      expect(narrow()).toBe(true);
      dispose();
    });
  });

  it('removes its listener when the owner is disposed', () => {
    const media = stubMatchMedia(false);
    createRoot((dispose) => {
      createNarrowViewport();
      expect(media.listeners.size).toBe(1);
      dispose();
    });
    expect(media.listeners.size).toBe(0);
  });
});

describe('focusListRow', () => {
  function row(tabindex: number): HTMLButtonElement {
    const el = document.createElement('button');
    el.className = 'list__row';
    el.tabIndex = tabindex;
    document.body.append(el);
    return el;
  }

  it('focuses the row that was open when it is still mounted', () => {
    const roving = row(0);
    const opened = row(-1);
    expect(focusListRow(opened)).toBe(true);
    expect(document.activeElement).toBe(opened);
    expect(document.activeElement).not.toBe(roving);
  });

  it('falls back to the roving-focus row when the opened row has unmounted', () => {
    const roving = row(0);
    const opened = row(-1);
    opened.remove();
    expect(focusListRow(opened)).toBe(true);
    expect(document.activeElement).toBe(roving);
  });

  it('reports false and leaves focus alone when the list has no row', () => {
    expect(focusListRow(null)).toBe(false);
    expect(document.activeElement).toBe(document.body);
  });
});
