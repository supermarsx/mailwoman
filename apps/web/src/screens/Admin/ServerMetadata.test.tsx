import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { ServerMetadata } from './ServerMetadata.tsx';
import type { AdminApi, UserSummary } from '../../state/slices/admin.ts';
import type { JmapFn } from '../../api/acl-types.ts';
import { renderInShell } from './shell/testkit.tsx';
import { expectNoAxeViolations, expectNoInlineStyle } from './shell/uxTestkit.ts';

const ALICE: UserSummary = {
  accountId: 'acc-1',
  username: 'alice',
  domain: 'example.com',
  quota: null,
  flags: { zeroAccess: false, forcePasswordChange: false, remoteCacheWipe: false, disabled: false },
};

function api(listUsers: () => Promise<UserSummary[]>): AdminApi {
  return { listUsers } as unknown as AdminApi;
}

const noJmap: JmapFn = vi.fn(async () => ({ methodResponses: [], sessionState: 's' }));

describe('Admin › Server metadata (page states)', () => {
  it('no account: the empty state offers Add a user, which opens the Users section', async () => {
    const { navigate } = renderInShell(() => <ServerMetadata api={api(async () => [])} jmap={noJmap} />);
    fireEvent.click(await screen.findByRole('button', { name: 'Add a user' }));
    expect(navigate).toHaveBeenCalledWith(expect.stringContaining('#/users'));
  });

  it('a failed account list shows the error with Try again, which reads it again', async () => {
    const listUsers = vi.fn().mockRejectedValueOnce(Object.assign(new Error('x'), { status: 500 })).mockResolvedValue([ALICE]);
    renderInShell(() => <ServerMetadata api={api(listUsers)} jmap={noJmap} />);
    expect(await screen.findByText(/HTTP 500/)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Try again' }));
    expect(await screen.findByRole('combobox', { name: 'Account' })).toBeInTheDocument();
    expect(listUsers).toHaveBeenCalledTimes(2);
  });

  it('the picker has help with a Learn more link, and no axe violations or inline style', async () => {
    const { container } = renderInShell(() => <ServerMetadata api={api(async () => [ALICE])} jmap={noJmap} />);
    const picker = await screen.findByRole('combobox', { name: 'Account' });
    expect(picker).toHaveAccessibleDescription(/decides what the account may set/);
    expect(screen.getByRole('link', { name: 'Learn more about Account' }).getAttribute('href')).toMatch(/docs\/admin\/server-metadata\.md#account$/);
    await waitFor(() => expect(screen.getByRole('option', { name: 'alice@example.com' })).toBeInTheDocument());
    await expectNoAxeViolations(container);
    expectNoInlineStyle(container);
  });
});
