import { describe, it, expect, vi } from 'vitest';
import { render, waitFor, screen } from '@solidjs/testing-library';
import type { JmapRequest, JmapResponse } from '../api/jmap-types.ts';

// Mount smoke (t13 26.13 E9): prove the sharing dialog wires the PRODUCTION ACL
// client end-to-end — `createConfiguredClient().jmap` → `createAclClient` →
// `<AclEditor>` — and that the reconciled request rides only the JMAP core
// capability (no invented ACL URN). The transport is mocked to a controllable fake.
const { jmap } = vi.hoisted(() => {
  const jmap = vi.fn(async (body: JmapRequest): Promise<JmapResponse> => {
    const [name, , callId] = body.methodCalls[0]!;
    const arg =
      name === 'MailboxRights/get'
        ? { accountId: 'acc', myRights: 'lra', acl: [{ identifier: 'bob', rights: 'lr' }] }
        : { accountId: 'acc' };
    return { methodResponses: [[name, arg, callId]], sessionState: 's0' };
  });
  return { jmap };
});

vi.mock('../api/transport.ts', () => ({
  createConfiguredClient: () => ({ jmap }),
}));

import { SharingDialog } from './SharingDialog.tsx';

describe('SharingDialog mounts the ACL editor over the production client', () => {
  it('renders the editor and drives MailboxRights/get through the wired client', async () => {
    render(() => (
      <SharingDialog mailboxId="mbx1" accountId="acc" mailboxName="Team" onClose={() => {}} />
    ));

    // The E8 editor is mounted.
    expect(screen.getByTestId('acl-editor')).toBeTruthy();

    // The wired client issued MailboxRights/get for this mailbox + account.
    await waitFor(() => expect(jmap).toHaveBeenCalled());
    const body = jmap.mock.calls[0]![0];
    expect(body.methodCalls[0]![0]).toBe('MailboxRights/get');
    expect(body.methodCalls[0]![1]).toMatchObject({ accountId: 'acc', mailboxId: 'mbx1' });
    // Reconcile: `using` carries only the JMAP core capability — no invented URN.
    expect(body.using).toEqual(['urn:ietf:params:jmap:core']);

    // MYRIGHTS contained `a`, so the admin-gated grant form is exposed (the editor
    // received a live, correctly-parsed response through the mount wiring).
    await waitFor(() => expect(screen.getByTestId('add-grant-form')).toBeTruthy());
  });
});
