// t79: before the reader catalog loads, `t` answers with the message id; the
// level label must never show `reader-zoom-level`.
import { describe, expect, it, vi } from 'vitest';

const loaded = vi.hoisted(() => ({ value: false }));
vi.mock('../i18n/index.ts', async (original) => ({
  ...(await original<typeof import('../i18n/index.ts')>()),
  t: (id: string, args?: Record<string, unknown>) => (loaded.value && id === 'reader-zoom-level' ? `${String(args?.['level'])} %` : id),
}));
const { levelText } = await import('./ReaderZoom.tsx');

describe('zoom level label', () => {
  it('shows the plain percentage until the catalog has loaded, then the formatted one', () => {
    expect(levelText(100)).toBe('100%');
    expect(levelText(125)).toBe('125%');
    loaded.value = true;
    expect(levelText(125)).toBe('125 %');
  });
});
