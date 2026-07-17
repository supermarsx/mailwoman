import { describe, it, expect, vi } from 'vitest';
import { render, fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { AclEditor } from './AclEditor.tsx';
import {
  parseRights,
  serializeRights,
  toggleRight,
  hasRight,
  canAdminister,
  type AclClient,
  type MailboxRights,
} from '../../api/acl-types.ts';

// ── pure RFC 4314 rights helpers ─────────────────────────────────────────────

describe('RFC 4314 rights-bit helpers', () => {
  it('parses only recognised bits and ignores junk', () => {
    expect([...parseRights('lrs')].sort()).toEqual(['l', 'r', 's']);
    // unknown chars (RFC 4314 obsolete `c`/`d` or noise) are dropped
    expect([...parseRights('lrZ9')].sort()).toEqual(['l', 'r']);
  });

  it('serialises to canonical RFC order regardless of input order', () => {
    expect(serializeRights(['a', 'l', 'r'])).toBe('lra');
    expect(serializeRights(['e', 't', 'w'])).toBe('wte');
  });

  it('toggles a bit on and off, preserving the rest in canonical order', () => {
    expect(toggleRight('lr', 'w', true)).toBe('lrw');
    expect(toggleRight('lrw', 'r', false)).toBe('lw');
    // toggling an already-present bit on is idempotent
    expect(toggleRight('lr', 'l', true)).toBe('lr');
  });

  it('gates admin on the `a` right', () => {
    expect(canAdminister('lra')).toBe(true);
    expect(canAdminister('lrswipdxte')).toBe(false);
    expect(hasRight('lr', 'r')).toBe(true);
    expect(hasRight('lr', 'a')).toBe(false);
  });
});

// ── editor ───────────────────────────────────────────────────────────────────

function makeClient(rights: MailboxRights): {
  client: AclClient;
  grant: ReturnType<typeof vi.fn>;
  revoke: ReturnType<typeof vi.fn>;
} {
  const grant = vi.fn(async () => {});
  const revoke = vi.fn(async () => {});
  const client: AclClient = {
    getMailboxRights: async () => rights,
    grant,
    revoke,
    getServerMetadata: async () => [],
    setServerMetadata: async () => {},
    removeServerMetadata: async () => {},
  };
  return { client, grant, revoke };
}

describe('ACL editor — rights-bit checkbox mapping', () => {
  it('checks exactly the bits an entry holds', async () => {
    const { client } = makeClient({
      myRights: 'lra',
      acl: [{ identifier: 'bob', rights: 'lr' }],
    });
    render(() => <AclEditor mailboxId="mbx1" client={client} />);

    await waitFor(() => expect(screen.getByTestId('acl-entry')).toBeInTheDocument());
    expect(screen.getByTestId('acl-bob-l')).toBeChecked();
    expect(screen.getByTestId('acl-bob-r')).toBeChecked();
    expect(screen.getByTestId('acl-bob-w')).not.toBeChecked();
    expect(screen.getByTestId('acl-bob-a')).not.toBeChecked();
  });
});

describe('ACL editor — read-only unless the current user holds `a`', () => {
  it('renders read-only (no write controls, disabled checkboxes) without `a`', async () => {
    const { client } = makeClient({
      myRights: 'lr', // no admin right
      acl: [{ identifier: 'bob', rights: 'lr' }],
    });
    render(() => <AclEditor mailboxId="mbx1" client={client} />);

    await waitFor(() => expect(screen.getByTestId('readonly-notice')).toBeInTheDocument());
    expect(screen.getByTestId('acl-bob-l')).toBeDisabled();
    expect(screen.queryByTestId('add-grant-form')).not.toBeInTheDocument();
    expect(screen.queryByTestId('remove-grant')).not.toBeInTheDocument();
  });

  it('enables write controls when the user holds `a`', async () => {
    const { client } = makeClient({
      myRights: 'lra',
      acl: [{ identifier: 'bob', rights: 'lr' }],
    });
    render(() => <AclEditor mailboxId="mbx1" client={client} />);

    await waitFor(() => expect(screen.getByTestId('add-grant-form')).toBeInTheDocument());
    expect(screen.getByTestId('acl-bob-l')).toBeEnabled();
    expect(screen.getByTestId('remove-grant')).toBeInTheDocument();
    expect(screen.queryByTestId('readonly-notice')).not.toBeInTheDocument();
  });
});

describe('ACL editor — toggle / add / remove grant flows', () => {
  it('toggling a bit on an existing entry calls grant with the new canonical rights', async () => {
    const { client, grant } = makeClient({
      myRights: 'lra',
      acl: [{ identifier: 'bob', rights: 'lr' }],
    });
    render(() => <AclEditor mailboxId="mbx1" client={client} />);

    await waitFor(() => expect(screen.getByTestId('acl-bob-w')).toBeInTheDocument());
    fireEvent.change(screen.getByTestId('acl-bob-w'), { target: { checked: true } });

    await waitFor(() => expect(grant).toHaveBeenCalledTimes(1));
    expect(grant).toHaveBeenCalledWith('mbx1', 'bob', 'lrw');
  });

  it('unchecking a bit calls grant with that bit removed', async () => {
    const { client, grant } = makeClient({
      myRights: 'lra',
      acl: [{ identifier: 'bob', rights: 'lrw' }],
    });
    render(() => <AclEditor mailboxId="mbx1" client={client} />);

    await waitFor(() => expect(screen.getByTestId('acl-bob-r')).toBeChecked());
    fireEvent.change(screen.getByTestId('acl-bob-r'), { target: { checked: false } });

    await waitFor(() => expect(grant).toHaveBeenCalledWith('mbx1', 'bob', 'lw'));
  });

  it('adds a new grant from the form with the selected bits', async () => {
    const { client, grant } = makeClient({ myRights: 'a', acl: [] });
    render(() => <AclEditor mailboxId="mbx1" client={client} />);

    await waitFor(() => expect(screen.getByTestId('add-grant-form')).toBeInTheDocument());
    fireEvent.input(screen.getByTestId('new-identifier'), { target: { value: 'alice' } });
    fireEvent.change(screen.getByTestId('acl-new-l'), { target: { checked: true } });
    fireEvent.change(screen.getByTestId('acl-new-r'), { target: { checked: true } });
    fireEvent.click(screen.getByTestId('submit-grant'));

    await waitFor(() => expect(grant).toHaveBeenCalledWith('mbx1', 'alice', 'lr'));
  });

  it('does not submit an empty identifier', async () => {
    const { client, grant } = makeClient({ myRights: 'a', acl: [] });
    render(() => <AclEditor mailboxId="mbx1" client={client} />);

    await waitFor(() => expect(screen.getByTestId('submit-grant')).toBeInTheDocument());
    // button disabled with an empty identifier
    expect(screen.getByTestId('submit-grant')).toBeDisabled();
    expect(grant).not.toHaveBeenCalled();
  });

  it('removing an entry calls revoke', async () => {
    const { client, revoke } = makeClient({
      myRights: 'lra',
      acl: [{ identifier: 'bob', rights: 'lr' }],
    });
    render(() => <AclEditor mailboxId="mbx1" client={client} />);

    await waitFor(() => expect(screen.getByTestId('remove-grant')).toBeInTheDocument());
    fireEvent.click(screen.getByTestId('remove-grant'));

    await waitFor(() => expect(revoke).toHaveBeenCalledWith('mbx1', 'bob'));
  });
});
