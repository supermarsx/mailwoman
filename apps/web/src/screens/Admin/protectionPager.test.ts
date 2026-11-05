import { describe, it, expect, vi } from 'vitest';
import { createRoot } from 'solid-js';
import { createPager, DEFAULT_PAGE_SIZE, PAGE_SIZES } from './protectionPager.ts';

describe('createPager', () => {
  it('defaults to 100 rows and offers 25/50/100/250', () => {
    expect(DEFAULT_PAGE_SIZE).toBe(100);
    expect(PAGE_SIZES).toEqual([25, 50, 100, 250]);
  });
  it('remembers cursors so Previous returns to the same newest-first page', async () => {
    const fetchPage = vi.fn(async (after: string | undefined) => (after === 'b' ? { items: [3], cursor: null, total: 3 } : after === 'a' ? { items: [2], cursor: 'b', total: 3 } : { items: [1], cursor: 'a', total: 3 }));
    await createRoot(async (dispose) => {
      const pager = createPager<number>(fetchPage);
      await pager.reload();
      expect(pager.items()).toEqual([1]);
      await pager.nextPage(); await pager.nextPage();
      expect([pager.page(), pager.items(), pager.hasNext()]).toEqual([3, [3], false]);
      await pager.prevPage();
      expect([pager.page(), pager.items(), pager.hasPrev()]).toEqual([2, [2], true]);
      await pager.setSize(25);
      expect(fetchPage).toHaveBeenLastCalledWith(undefined, 25);
      expect(pager.page()).toBe(1);
      dispose();
    });
  });
  it('steps back when the current page empties', async () => {
    let rows = true;
    const pager = createRoot(() => createPager<number>(async (after) => (after ? { items: rows ? [2] : [], cursor: null, total: rows ? 2 : 1 } : { items: [1], cursor: 'a', total: 2 })));
    await pager.reload(); await pager.nextPage();
    rows = false;
    await pager.reload();
    expect(pager.page()).toBe(1);
    expect(pager.items()).toEqual([1]);
  });
});
