import { describe, it, expect, vi } from 'vitest';
import { createSubTabs } from './subTabs.ts';

describe('createSubTabs', () => {
  it('opens a tab, makes it active, and returns its id', () => {
    const s = createSubTabs();
    const id = s.open({ kind: 'messages', title: 'Inbox' });
    expect(s.tabs()).toHaveLength(1);
    expect(s.activeId()).toBe(id);
    expect(s.tabs()[0]!.title).toBe('Inbox');
  });

  it('reuses an existing tab by id instead of duplicating', () => {
    const s = createSubTabs();
    const a = s.open({ id: 'draft-1', kind: 'composer', title: 'Draft' });
    s.open({ kind: 'messages', title: 'Inbox' });
    const b = s.open({ id: 'draft-1', kind: 'composer', title: 'Draft' });
    expect(a).toBe(b);
    expect(s.tabs()).toHaveLength(2);
    expect(s.activeId()).toBe('draft-1');
  });

  it('closing the active tab focuses the left neighbour', () => {
    const s = createSubTabs();
    const a = s.open({ kind: 'messages', title: 'A' });
    const b = s.open({ kind: 'messages', title: 'B' });
    const c = s.open({ kind: 'messages', title: 'C' });
    s.activate(b);
    s.close(b);
    expect(s.tabs().map((t) => t.id)).toEqual([a, c]);
    expect(s.activeId()).toBe(a);
  });

  it('closing the last remaining tab clears the active id', () => {
    const s = createSubTabs();
    const a = s.open({ kind: 'settings', title: 'Settings' });
    s.close(a);
    expect(s.tabs()).toHaveLength(0);
    expect(s.activeId()).toBeNull();
  });

  it('togglePin flips the pinned flag', () => {
    const s = createSubTabs();
    const a = s.open({ kind: 'messages', title: 'A' });
    expect(s.tabs()[0]!.pinned).toBe(false);
    s.togglePin(a);
    expect(s.tabs()[0]!.pinned).toBe(true);
  });

  it('cycle moves focus and wraps in both directions', () => {
    const s = createSubTabs();
    const a = s.open({ kind: 'messages', title: 'A' });
    const b = s.open({ kind: 'messages', title: 'B' });
    s.activate(a);
    expect(s.cycle(1)).toBe(b);
    expect(s.cycle(1)).toBe(a); // wrap forward
    expect(s.cycle(-1)).toBe(b); // wrap backward
  });

  it('cycle returns null when there are no tabs', () => {
    const s = createSubTabs();
    expect(s.cycle(1)).toBeNull();
  });

  it('tearOff opens a window with the tab url and closes it locally', () => {
    const openWindow = vi.fn(() => ({}));
    const s = createSubTabs({ openWindow });
    const a = s.open({ id: 'm-1', kind: 'messages', title: 'Inbox' });
    s.tearOff(a);
    expect(openWindow).toHaveBeenCalledWith('?tab=m-1', '_blank');
    expect(s.tabs()).toHaveLength(0);
  });
});
