import { describe, it, expect, beforeEach } from 'vitest';
import { screen, waitFor } from '@solidjs/testing-library';
import { MessageList } from './MessageList.tsx';
import { renderWithApp, mkEmail } from './appHarness.tsx';

describe('MessageList (virtualized)', () => {
  beforeEach(() => localStorage.clear());

  it('mounts only a window of rows for a huge list', async () => {
    const emails = Array.from({ length: 5000 }, (_, i) =>
      mkEmail(`m${i}`, { subject: `Message ${i}` }),
    );
    const { app, result } = renderWithApp(() => <MessageList />, { emails });
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });

    await waitFor(() => expect(app.messages().length).toBe(5000));
    // Only the viewport window (± overscan) is in the DOM, not all 5000 rows.
    const rows = result.container.querySelectorAll('.list__row');
    expect(rows.length).toBeGreaterThan(0);
    expect(rows.length).toBeLessThan(40);
    // The spacer still reflects the full list height so the scrollbar is honest.
    const items = result.container.querySelector('.list__items') as HTMLElement;
    expect(items.style.height).toBe(`${5000 * 72}px`);
  });

  it('renders tag chips from the registry for a labeled message', async () => {
    const { app } = renderWithApp(() => <MessageList />, {
      emails: [mkEmail('a', { keywords: { $seen: true, work: true } })],
    });
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    await waitFor(() => expect(app.messages().length).toBe(1));
    // 'work' is a seeded registry tag → its display name renders as a chip.
    expect(await screen.findByText('Work')).toBeInTheDocument();
  });

  it('exposes list semantics: each row announces its position in the full list', async () => {
    const emails = Array.from({ length: 50 }, (_, i) => mkEmail(`m${i}`, { subject: `Message ${i}` }));
    const { app, result } = renderWithApp(() => <MessageList />, { emails });
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    await waitFor(() => expect(app.messages().length).toBe(50));

    const firstItem = result.container.querySelector('.list__slot') as HTMLElement;
    expect(firstItem.getAttribute('role')).toBe('listitem');
    expect(firstItem.getAttribute('aria-setsize')).toBe('50');
    expect(firstItem.getAttribute('aria-posinset')).toBe('1');
    // Roving tabindex: the cursor row is the single tab stop.
    const firstRow = firstItem.querySelector('.list__row') as HTMLElement;
    expect(firstRow.getAttribute('tabindex')).toBe('0');
  });

  it('floats a pinned message to the top row', async () => {
    const { app, result } = renderWithApp(() => <MessageList />, {
      emails: [mkEmail('a', { subject: 'Alpha' }), mkEmail('b', { subject: 'Bravo', pinned: true })],
    });
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    await waitFor(() => expect(app.messages().length).toBe(2));
    const firstRow = result.container.querySelector('.list__row') as HTMLElement;
    expect(firstRow.textContent).toContain('Bravo');
  });
});
