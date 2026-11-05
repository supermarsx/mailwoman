import { createSignal } from 'solid-js';
import { describeAdminError } from './shell/Feedback.tsx';

/** Page sizes an admin can pick; the first load uses {@link DEFAULT_PAGE_SIZE}. */
export const PAGE_SIZES = [25, 50, 100, 250] as const;
export const DEFAULT_PAGE_SIZE = 100;

export interface PagedResult<T> { items: T[]; cursor: string | null; total: number }

/**
 * Cursor paging with Previous/Next on top of a forward-only server cursor: the
 * cursor that opened every visited page is remembered, so Previous re-requests
 * the earlier page (newest first, stable ordering) instead of guessing offsets.
 */
export function createPager<T>(fetchPage: (after: string | undefined, limit: number) => Promise<PagedResult<T>>) {
  const [items, setItems] = createSignal<T[]>([]);
  const [total, setTotal] = createSignal(0);
  const [loading, setLoading] = createSignal(false);
  const [loaded, setLoaded] = createSignal(false);
  const [error, setError] = createSignal('');
  const [size, setSizeSignal] = createSignal<number>(DEFAULT_PAGE_SIZE);
  const [stack, setStack] = createSignal<(string | undefined)[]>([undefined]);
  const [next, setNext] = createSignal<string | null>(null);
  const index = (): number => stack().length - 1;

  async function load(): Promise<void> {
    setLoading(true);
    setError('');
    try {
      const result = await fetchPage(stack()[index()], size());
      if (result.items.length === 0 && index() > 0) {
        // The page emptied (rows expired or were cleared): step back one page.
        setStack(stack().slice(0, -1));
        return await load();
      }
      setItems(result.items);
      setNext(result.cursor);
      setTotal(result.total);
      setLoaded(true);
    } catch (cause) {
      setError(describeAdminError(cause));
    } finally {
      setLoading(false);
    }
  }
  return {
    items, total, loading, loaded, error, size,
    page: (): number => index() + 1,
    pages: (): number => Math.max(1, Math.ceil(total() / size())),
    hasPrev: (): boolean => index() > 0,
    hasNext: (): boolean => next() !== null,
    reload: load,
    first: async (): Promise<void> => { setStack([undefined]); await load(); },
    async nextPage(): Promise<void> { const cursor = next(); if (!cursor) return; setStack([...stack(), cursor]); await load(); },
    async prevPage(): Promise<void> { if (index() === 0) return; setStack(stack().slice(0, -1)); await load(); },
    async setSize(value: number): Promise<void> { setSizeSignal(value); setStack([undefined]); await load(); },
  };
}
