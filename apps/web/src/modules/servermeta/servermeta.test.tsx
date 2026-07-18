import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { MetadataView } from './MetadataView.tsx';
import { ServerMetadata } from '../../screens/Admin/ServerMetadata.tsx';
import { createAclClient, type AclClient, type MetadataEntry } from '../../api/acl-types.ts';
import type { JmapRequest, JmapResponse } from '../../api/jmap-types.ts';
import type { AdminApi, UserSummary } from '../../state/slices/admin.ts';

function makeClient(entries: MetadataEntry[]): {
  client: AclClient;
  set: ReturnType<typeof vi.fn>;
  remove: ReturnType<typeof vi.fn>;
  get: ReturnType<typeof vi.fn>;
} {
  const set = vi.fn(async () => {});
  const remove = vi.fn(async () => {});
  const get = vi.fn(async () => entries);
  const client: AclClient = {
    getMailboxRights: async () => ({ myRights: '', acl: [] }),
    grant: async () => {},
    revoke: async () => {},
    getServerMetadata: get,
    setServerMetadata: set,
    removeServerMetadata: remove,
  };
  return { client, set, remove, get };
}

describe('metadata view — read-only listing (default)', () => {
  it('lists entries with values and shows the read-only notice, no edit controls', async () => {
    const { client } = makeClient([
      { entry: '/shared/comment', value: 'hello' },
      { entry: '/shared/admin', value: null },
    ]);
    const { container } = render(() => <MetadataView client={client} />);

    await waitFor(() => expect(screen.getAllByTestId('metadata-entry')).toHaveLength(2));
    // the entry path is bidi-isolated in the DOM, so match on the data attribute
    expect(container.querySelector('[data-entry="/shared/comment"]')).toBeInTheDocument();
    expect(screen.getByText('hello')).toBeInTheDocument();
    // an unset value renders the "Not set" placeholder, not an empty node
    expect(screen.getByText('Not set')).toBeInTheDocument();
    expect(screen.getByTestId('readonly-notice')).toBeInTheDocument();
    expect(screen.queryByTestId('value-input')).not.toBeInTheDocument();
    expect(screen.queryByTestId('add-entry-form')).not.toBeInTheDocument();
  });

  it('passes null scope for server-level and the mailbox id for a mailbox scope', async () => {
    const server = makeClient([]);
    render(() => <MetadataView client={server.client} />);
    await waitFor(() => expect(server.get).toHaveBeenCalledWith(null));

    const box = makeClient([]);
    render(() => <MetadataView client={box.client} mailboxId="mbx7" />);
    await waitFor(() => expect(box.get).toHaveBeenCalledWith('mbx7'));
  });
});

describe('metadata view — guarded edit (canEdit)', () => {
  it('exposes edit + remove + add controls when canEdit', async () => {
    const { client } = makeClient([{ entry: '/shared/comment', value: 'hello' }]);
    render(() => <MetadataView client={client} canEdit />);

    await waitFor(() => expect(screen.getByTestId('add-entry-form')).toBeInTheDocument());
    expect(screen.getByTestId('value-input')).toBeInTheDocument();
    expect(screen.getByTestId('save-entry')).toBeInTheDocument();
    expect(screen.getByTestId('remove-entry')).toBeInTheDocument();
    expect(screen.queryByTestId('readonly-notice')).not.toBeInTheDocument();
  });

  it('saving an edited value calls setServerMetadata', async () => {
    const { client, set } = makeClient([{ entry: '/shared/comment', value: 'hello' }]);
    render(() => <MetadataView client={client} mailboxId="mbx1" canEdit />);

    await waitFor(() => expect(screen.getByTestId('value-input')).toBeInTheDocument());
    fireEvent.input(screen.getByTestId('value-input'), { target: { value: 'updated' } });
    fireEvent.click(screen.getByTestId('save-entry'));

    await waitFor(() => expect(set).toHaveBeenCalledWith('mbx1', '/shared/comment', 'updated'));
  });

  it('removing an entry calls removeServerMetadata', async () => {
    const { client, remove } = makeClient([{ entry: '/shared/comment', value: 'hello' }]);
    render(() => <MetadataView client={client} canEdit />);

    await waitFor(() => expect(screen.getByTestId('remove-entry')).toBeInTheDocument());
    fireEvent.click(screen.getByTestId('remove-entry'));

    await waitFor(() => expect(remove).toHaveBeenCalledWith(null, '/shared/comment'));
  });

  it('adds a new annotation from the form', async () => {
    const { client, set } = makeClient([]);
    render(() => <MetadataView client={client} canEdit />);

    await waitFor(() => expect(screen.getByTestId('add-entry-form')).toBeInTheDocument());
    fireEvent.input(screen.getByTestId('new-entry'), { target: { value: '/shared/comment' } });
    fireEvent.input(screen.getByTestId('new-value'), { target: { value: 'note' } });
    fireEvent.click(screen.getByTestId('submit-entry'));

    await waitFor(() => expect(set).toHaveBeenCalledWith(null, '/shared/comment', 'note'));
  });
});

// ── createAclClient — request building + response parsing over a fake jmap ────

describe('createAclClient wires the JMAP method surface', () => {
  function fakeJmap(): { jmap: ReturnType<typeof vi.fn>; calls: JmapRequest[] } {
    const calls: JmapRequest[] = [];
    const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
      calls.push(body);
      const [name, , callId] = body.methodCalls[0]!;
      const arg =
        name === 'MailboxRights/get'
          ? { accountId: 'acc', myRights: 'lra', acl: [{ identifier: 'bob', rights: 'lr' }] }
          : name === 'ServerMetadata/get'
            ? { accountId: 'acc', list: [{ entry: '/shared/comment', value: 'hi' }] }
            : { accountId: 'acc' };
      return { methodResponses: [[name, arg, callId]], sessionState: 's0' };
    });
    return { jmap, calls };
  }

  it('getMailboxRights builds MailboxRights/get and parses myRights + acl', async () => {
    const { jmap, calls } = fakeJmap();
    const client = createAclClient('acc', jmap);
    const rights = await client.getMailboxRights('mbx1');

    expect(calls[0]!.methodCalls[0]![0]).toBe('MailboxRights/get');
    expect(calls[0]!.methodCalls[0]![1]).toMatchObject({ accountId: 'acc', mailboxId: 'mbx1' });
    expect(rights.myRights).toBe('lra');
    expect(rights.acl).toEqual([{ identifier: 'bob', rights: 'lr' }]);
  });

  it('grant builds MailboxRights/set with rights; revoke sends null (DELETEACL, not SETACL-empty)', async () => {
    const { jmap, calls } = fakeJmap();
    const client = createAclClient('acc', jmap);
    await client.grant('mbx1', 'alice', 'lr');
    await client.revoke('mbx1', 'alice');

    expect(calls[0]!.methodCalls[0]![1]).toMatchObject({ mailboxId: 'mbx1', identifier: 'alice', rights: 'lr' });
    // E9 reconcile: E7 maps null/absent rights → DELETEACL; an empty string would
    // be a SETACL to empty rights, so revoke must send null.
    expect(calls[1]!.methodCalls[0]![1]).toMatchObject({ mailboxId: 'mbx1', identifier: 'alice', rights: null });
  });

  it('getServerMetadata builds ServerMetadata/get (null scope) and parses the list', async () => {
    const { jmap, calls } = fakeJmap();
    const client = createAclClient('acc', jmap);
    const list = await client.getServerMetadata(null);

    expect(calls[0]!.methodCalls[0]![0]).toBe('ServerMetadata/get');
    expect(calls[0]!.methodCalls[0]![1]).toMatchObject({ mailboxId: null });
    expect(list).toEqual([{ entry: '/shared/comment', value: 'hi' }]);
  });

  it('set / remove build ServerMetadata/set (remove = null value)', async () => {
    const { jmap, calls } = fakeJmap();
    const client = createAclClient('acc', jmap);
    await client.setServerMetadata('mbx1', '/shared/comment', 'x');
    await client.removeServerMetadata('mbx1', '/shared/comment');

    expect(calls[0]!.methodCalls[0]![1]).toMatchObject({ entry: '/shared/comment', value: 'x' });
    expect(calls[1]!.methodCalls[0]![1]).toMatchObject({ entry: '/shared/comment', value: null });
  });
});

// ── Admin mount (t14 E4): write-capable editor behind an account picker ───────

describe('admin server-metadata mount — write-capable behind an account picker', () => {
  function makeAdminApi(users: UserSummary[]): AdminApi {
    // Only `listUsers` is exercised by the wrapper; the rest are inert stubs.
    const api: Partial<AdminApi> = { listUsers: async () => users };
    return api as AdminApi;
  }

  function fakeJmap(): { jmap: ReturnType<typeof vi.fn>; calls: JmapRequest[] } {
    const calls: JmapRequest[] = [];
    const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
      calls.push(body);
      const [name, , callId] = body.methodCalls[0]!;
      const arg =
        name === 'ServerMetadata/get'
          ? { accountId: 'acc', list: [{ entry: '/shared/comment', value: 'hi' }] }
          : { accountId: 'acc' };
      return { methodResponses: [[name, arg, callId]], sessionState: 's0' };
    });
    return { jmap, calls };
  }

  const USERS: UserSummary[] = [
    {
      accountId: 'acc-1',
      username: 'alice',
      domain: 'example.com',
      quota: null,
      flags: { zeroAccess: false, forcePasswordChange: false, remoteCacheWipe: false, disabled: false },
    },
  ];

  it('prompts for an account and shows no editor until one is picked', async () => {
    const { jmap } = fakeJmap();
    render(() => <ServerMetadata api={makeAdminApi(USERS)} jmap={jmap} />);

    // the account picker is populated from listUsers()
    await waitFor(() => expect(screen.getByRole('option', { name: 'alice@example.com' })).toBeInTheDocument());
    // nothing is scoped yet → the view is not mounted and no JMAP call was made
    expect(screen.queryByTestId('metadata-view')).not.toBeInTheDocument();
    expect(jmap).not.toHaveBeenCalled();
  });

  it('mounts the write-capable MetadataView (canEdit) for the picked account', async () => {
    const { jmap, calls } = fakeJmap();
    render(() => <ServerMetadata api={makeAdminApi(USERS)} jmap={jmap} />);

    await waitFor(() => expect(screen.getByTestId('admin-servermeta-account')).toBeInTheDocument());
    fireEvent.change(screen.getByTestId('admin-servermeta-account'), { target: { value: 'acc-1' } });

    // the editor mounts with edit controls (canEdit) — add form + per-entry save
    // (the loaded entry row), no read-only notice
    await waitFor(() => expect(screen.getByTestId('add-entry-form')).toBeInTheDocument());
    await waitFor(() => expect(screen.getByTestId('save-entry')).toBeInTheDocument());
    expect(screen.queryByTestId('readonly-notice')).not.toBeInTheDocument();

    // the metadata load rode the injected transport, server-level scope (mailboxId null),
    // carrying the SELECTED account id — the passthrough contract E-mount fulfils.
    await waitFor(() => expect(calls.length).toBeGreaterThan(0));
    expect(calls[0]!.methodCalls[0]![0]).toBe('ServerMetadata/get');
    expect(calls[0]!.methodCalls[0]![1]).toMatchObject({ accountId: 'acc-1', mailboxId: null });
  });

  it('writes reach ServerMetadata/set for the selected account', async () => {
    const { jmap, calls } = fakeJmap();
    render(() => <ServerMetadata api={makeAdminApi(USERS)} jmap={jmap} />);

    await waitFor(() => expect(screen.getByTestId('admin-servermeta-account')).toBeInTheDocument());
    fireEvent.change(screen.getByTestId('admin-servermeta-account'), { target: { value: 'acc-1' } });

    await waitFor(() => expect(screen.getByTestId('add-entry-form')).toBeInTheDocument());
    fireEvent.input(screen.getByTestId('new-entry'), { target: { value: '/shared/comment' } });
    fireEvent.input(screen.getByTestId('new-value'), { target: { value: 'note' } });
    fireEvent.click(screen.getByTestId('submit-entry'));

    await waitFor(() => {
      const setCall = calls.find((c) => c.methodCalls[0]![0] === 'ServerMetadata/set');
      expect(setCall).toBeDefined();
      expect(setCall!.methodCalls[0]![1]).toMatchObject({
        accountId: 'acc-1',
        mailboxId: null,
        entry: '/shared/comment',
        value: 'note',
      });
    });
  });
});
