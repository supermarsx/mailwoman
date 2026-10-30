import { describe, it, expect, vi } from 'vitest';
import { COMMON_SECTIONS, SECTIONS, prefetchSectionsWhenIdle, preloadSection, type SectionDef } from './sections.ts';

function fake(id: string, preload: () => Promise<unknown>): SectionDef {
  const component = Object.assign(() => null, { preload });
  return { id, group: 'overview', icon: 'dashboard', label: () => id, component } as unknown as SectionDef;
}

describe('admin section prefetch (t99)', () => {
  it('names sections that exist', () => {
    for (const id of COMMON_SECTIONS) expect(SECTIONS.some((s) => s.id === id)).toBe(true);
  });

  it('preloadSection starts the lazy load and swallows its failure', async () => {
    const preload = vi.fn(async () => { throw new Error('chunk gone'); });
    preloadSection('a', [fake('a', preload)]);
    preloadSection('missing', [fake('a', preload)]);
    expect(preload).toHaveBeenCalledTimes(1);
    await Promise.resolve();
  });

  it('prefetches one section per idle period, in order', async () => {
    const idle: (() => void)[] = [];
    vi.stubGlobal('requestIdleCallback', (cb: () => void) => { idle.push(cb); return idle.length; });
    try {
      const order: string[] = [];
      const sections = ['a', 'b'].map((id) => fake(id, async () => { order.push(id); }));
      prefetchSectionsWhenIdle(['a', 'b'], sections);
      expect(order).toEqual([]);
      idle.shift()!();
      await new Promise((r) => setTimeout(r, 0));
      expect(order).toEqual(['a']);
      idle.shift()!();
      await new Promise((r) => setTimeout(r, 0));
      expect(order).toEqual(['a', 'b']);
    } finally {
      vi.unstubAllGlobals();
    }
  });
});
