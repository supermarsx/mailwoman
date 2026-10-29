import { describe, expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { renderWithApp } from './appHarness.tsx';
import { RemoteChangeNotice } from './RemoteChangeNotice.tsx';
import { createMessageView } from '../state/slices/messageView.ts';
import type { Email } from '../api/jmap-types.ts';

describe('the open message changed elsewhere (t39)', () => {
  it('names the folder it moved to, keeps the reader open, and can be dismissed', async () => {
    const view = createMessageView({ account: () => 'acct1', load: async (_a, id) => ({ email: { id, keywords: {}, mailboxIds: { inbox: true } } as unknown as Email, html: 'body' }), readDelay: () => null, markRead: vi.fn() });
    const { app } = renderWithApp(() => <RemoteChangeNotice view={view} />);
    await app.login({ username: 'me@example.org', password: 'p' });
    await view.openMessage('m');
    const region = screen.getByTestId('reader-remote-change-region');
    expect(region).toHaveAttribute('role', 'status');
    expect(screen.queryByTestId('reader-remote-change')).toBeNull();
    const target = app.mailboxes().find((box) => box.role === 'archive') ?? app.mailboxes().find((box) => box.role !== 'inbox')!;
    view.reconcile('m', { keywords: {}, mailboxIds: { [target.id]: true } });
    await waitFor(() => expect(screen.getByTestId('reader-remote-change')).toHaveTextContent(new RegExp(`moved to \\W?${target.name}\\W? from another device or app`)));
    expect(view.openEmail()?.id).toBe('m');
    fireEvent.click(screen.getByRole('button', { name: 'Dismiss' }));
    expect(screen.queryByTestId('reader-remote-change')).toBeNull();
    view.reconcile('m', null);
    await waitFor(() => expect(screen.getByTestId('reader-remote-change')).toHaveTextContent('deleted from another device or app'));
    view.dispose();
  });
});
