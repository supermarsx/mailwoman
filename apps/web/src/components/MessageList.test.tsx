import { describe, it, expect, beforeEach } from 'vitest';
import { screen, waitFor, fireEvent } from '@solidjs/testing-library';
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

  // ── W17 density → virtualized row height ──────────────────────────────────
  it('scales the virtualized row height with the density preference', async () => {
    const emails = Array.from({ length: 20 }, (_, i) => mkEmail(`m${i}`, { subject: `Message ${i}` }));
    const { app, result } = renderWithApp(() => <MessageList />, { emails });
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    await waitFor(() => expect(app.messages().length).toBe(20));

    const items = () => result.container.querySelector('.list__items') as HTMLElement;
    // Default density (cozy) keeps the historical 72px row.
    expect(items().style.height).toBe(`${20 * 72}px`);

    app.setDensity('compact');
    await waitFor(() => expect(items().style.height).toBe(`${20 * 56}px`));

    app.setDensity('relaxed');
    await waitFor(() => expect(items().style.height).toBe(`${20 * 88}px`));
  });

  // ── W2 conversation threading ─────────────────────────────────────────────
  it('folds a shared-threadId conversation into one expandable head row', async () => {
    const emails = [
      mkEmail('m3', { threadId: 't1', subject: 'Re: Hi', receivedAt: '2026-01-03T00:00:00Z' }),
      mkEmail('m2', { threadId: 't1', subject: 'Re: Hi', receivedAt: '2026-01-02T00:00:00Z' }),
      mkEmail('m1', { threadId: 't1', subject: 'Hi', receivedAt: '2026-01-01T00:00:00Z' }),
      mkEmail('solo', { subject: 'Alone' }),
    ];
    const { app, result } = renderWithApp(() => <MessageList />, { emails });
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    await waitFor(() => expect(app.messages().length).toBe(4));

    // One conversation head (rep = newest 'Re: Hi') + one singleton row.
    const heads = result.container.querySelectorAll('[data-testid="thread-head"]');
    expect(heads.length).toBe(1);
    // The oldest member's subject ('Hi') is hidden until the thread expands.
    expect(screen.queryByText('Hi')).toBeNull();

    fireEvent.click(screen.getByTestId('thread-toggle'));

    // Expanded: the member with subject 'Hi' is now mounted.
    await waitFor(() => expect(screen.queryByText('Hi')).not.toBeNull());
    expect(screen.getByTestId('thread-toggle').getAttribute('aria-expanded')).toBe('true');
  });

  it('leaves a thread-less list rendering exactly as a flat list (no thread heads)', async () => {
    const emails = Array.from({ length: 8 }, (_, i) => mkEmail(`m${i}`, { subject: `Message ${i}` }));
    const { app, result } = renderWithApp(() => <MessageList />, { emails });
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    await waitFor(() => expect(app.messages().length).toBe(8));
    expect(result.container.querySelectorAll('[data-testid="thread-head"]').length).toBe(0);
    expect(result.container.querySelectorAll('.list__row').length).toBe(8);
  });

  // ── W3 reading-pane position control ──────────────────────────────────────
  it('drives the reading-pane position from the list toolbar', async () => {
    const { app } = renderWithApp(() => <MessageList />, { emails: [mkEmail('a')] });
    await app.login({ jmapUrl: 'x', username: 'me@example.org', password: 'p' });
    await waitFor(() => expect(app.messages().length).toBe(1));

    const root = document.documentElement;
    fireEvent.click(screen.getByTestId('reading-pane-bottom'));
    expect(root.getAttribute('data-reading-pane')).toBe('bottom');
    expect(screen.getByTestId('reading-pane-bottom').getAttribute('aria-pressed')).toBe('true');

    fireEvent.click(screen.getByTestId('reading-pane-off'));
    expect(root.getAttribute('data-reading-pane')).toBe('off');

    // Reset to the default so the module singleton doesn't leak into other specs.
    fireEvent.click(screen.getByTestId('reading-pane-right'));
    expect(root.getAttribute('data-reading-pane')).toBe('right');
  });
});
