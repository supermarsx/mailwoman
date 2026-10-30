// t87-e4 (D11): Compose signs only with a key of the mailbox it composes for.
// A signing session kept across a mailbox switch is locked and dropped, and a
// key whose owner account is not the composing mailbox is refused.
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import { Compose } from './Compose.tsx';
import { makeClient } from './appHarness.tsx';
import { createAppState, type AppState } from '../state/store.ts';
import { AppContext } from '../state/context.ts';
import { clearSettings } from '../api/settings.ts';
import { assertSigningKeyOwner, SigningKeyOwnerError, type SigningSession } from './compose/crypto-jmap.ts';
import { __resetCryptoWorker, getCryptoWorker } from '../crypto/index.ts';
import type { CryptoKey } from '../api/crypto-types.ts';

function key(account: string): CryptoKey {
  return {
    id: `k-${account}`,
    kind: 'pgp',
    isOwn: true,
    addresses: ['me@example.org'], // ownership varies independently of the composing sender address
    fingerprint: `${account.toUpperCase()}FP`,
    keyId: `${account}id`,
    algorithm: 'ed25519',
    createdAt: '2026-01-01T00:00:00Z',
    expiresAt: null,
    publicKeyArmored: 'x',
    certPem: null,
    trust: 'verified',
    autocrypt: true,
    source: 'generated',
    hasPrivate: true,
    encryptedPrivateBackup: `BUNDLE-${account}`,
    verifiedAt: null,
    keyHistory: [],
  };
}

const session = (accountId: string): SigningSession => ({ keyRef: 'r', bundle: 'B', passphrase: 'p', accountId });

describe('assertSigningKeyOwner', () => {
  it('accepts a key of the composing mailbox', () => {
    expect(() => assertSigningKeyOwner(session('child'), 'child')).not.toThrow();
  });
  it('refuses a key of another mailbox, no composing mailbox, or no recorded owner', () => {
    expect(() => assertSigningKeyOwner(session('parent'), 'child')).toThrow(SigningKeyOwnerError);
    expect(() => assertSigningKeyOwner(session('parent'), null)).toThrow(SigningKeyOwnerError);
    expect(() => assertSigningKeyOwner(session(''), '')).toThrow(SigningKeyOwnerError);
  });
});

describe('Compose signing session across a mailbox switch (t87 D11)', () => {
  beforeAll(async () => {
    await import('./compose/RichTextEditor.tsx');
  }, 30_000);
  beforeEach(() => {
    localStorage.clear();
    __resetCryptoWorker();
  });
  afterEach(() => clearSettings());

  /** A real app whose mailbox and keys the test controls. */
  async function harness(initial = 'parent', initialOwner = initial) {
    const client = makeClient();
    const real = createAppState(client);
    await real.login({ username: 'me@example.org', password: 'p' });
    const [account] = createSignal<string | null>(initial);
    const [epoch, setEpoch] = createSignal(0);
    const [held, setHeld] = createSignal<CryptoKey[]>([key(initialOwner)]);
    const owners = new Map<CryptoKey, string>();
    owners.set(held()[0]!, initialOwner);
    let refs = 0;
    const unlockOwnKey = vi.fn(async (k: CryptoKey, _passphrase: string) => ({ keyRef: `ref-${owners.get(k)}-${++refs}`, accountId: owners.get(k)! }));
    const lockKeyRef = vi.fn(async () => undefined);
    const app = {
      ...real,
      accountId: account,
      ownKeys: held,
      loadKeys: vi.fn(async () => undefined),
      keyOwnerAccount: (k: CryptoKey) => (owners.get(k) === account() ? owners.get(k)! : null),
      unlockOwnKey,
      lockKeyRef,
      keysEpoch: epoch,
    } as unknown as AppState;
    render(() => (
      <AppContext.Provider value={app}>
        <Compose onClose={() => undefined} />
      </AppContext.Provider>
    ));
    /**
     * The keys side of a switch: the keys slice resets (a new epoch) and holds
     * the next mailbox's keys. (A change of `accountId` also closes an open
     * composer, see the draft-ownership effect in Compose; the signing session
     * must not outlive the keys reset even when the composer stays.)
     */
    const resetKeys = (nextKeys: CryptoKey[], owner: string): void => {
      for (const k of nextKeys) owners.set(k, owner);
      setHeld(nextKeys);
      setEpoch((n) => n + 1);
    };
    return { client, unlockOwnKey, lockKeyRef, resetKeys };
  }

  function sends(client: ReturnType<typeof makeClient>): number {
    return vi.mocked(client.jmap).mock.calls.filter((c) => c[0].methodCalls.some((m) => m[0] === 'EmailSubmission/set')).length;
  }

  async function unlock(passphrase: string): Promise<void> {
    fireEvent.input(await screen.findByTestId('sign-passphrase'), { target: { value: passphrase } });
    fireEvent.click(screen.getByTestId('sign-unlock-submit'));
  }

  function fill(): void {
    fireEvent.change(screen.getByTestId('format-toggle'), {target:{value:(screen.getByTestId('format-toggle') as HTMLSelectElement).value==='plain'?'rich':'plain'}});
    fireEvent.input(screen.getByLabelText('To'), { target: { value: 'you@example.org' } });
    fireEvent.input(screen.getByLabelText('Subject'), { target: { value: 'Hi' } });
    fireEvent.input(screen.getByLabelText('Body'), { target: { value: 'Hello' } });
  }

  it('locks and drops the unlocked signing key on a keys reset; a signed send then asks again', async () => {
    const h = await harness();
    const sign = vi.spyOn(getCryptoWorker(), 'sign');
    fill();
    fireEvent.click(await screen.findByTestId('sign-toggle'));
    await unlock('parent-pass');
    await waitFor(() => expect(h.unlockOwnKey).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(screen.queryByTestId('compose-sign-unlock')).toBeNull());

    // Same account id here only so the composer stays open; the owner map
    // re-tags the fresh key set for it.
    h.resetKeys([key('child')], 'parent');
    await waitFor(() => expect(h.lockKeyRef).toHaveBeenCalledWith('ref-parent-1'));

    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    // No signature with the parent key, nothing sent: the child's prompt opens.
    expect(await screen.findByTestId('compose-sign-unlock')).toBeInTheDocument();
    expect((screen.getByTestId('sign-passphrase') as HTMLInputElement).value).toBe('');
    expect(sign).not.toHaveBeenCalled();
    expect(sends(h.client)).toBe(0);

    await unlock('child-pass');
    await waitFor(() => expect(h.unlockOwnKey).toHaveBeenCalledTimes(2));
    expect(h.unlockOwnKey.mock.calls[1]![0].fingerprint).toBe('CHILDFP');
    expect(h.unlockOwnKey.mock.calls[1]![0].encryptedPrivateBackup).toBe('BUNDLE-child');
    expect(h.unlockOwnKey.mock.calls[1]![1]).toBe('child-pass');
  });

  it('refuses an owned key that belongs to a different sending address',async()=>{
    const h=await harness();h.resetKeys([{...key('parent'),addresses:['different@example.org']}],'parent');
    fill();fireEvent.click(await screen.findByTestId('sign-toggle'));await unlock('pw');
    expect(await screen.findByRole('alert')).toHaveTextContent('No signing key');
    expect(h.unlockOwnKey).not.toHaveBeenCalled();expect(sends(h.client)).toBe(0);
  });

  it('refuses to unlock a key whose owner is not the composing mailbox', async () => {
    // The child is open, but the only key offered still belongs to the parent.
    const h = await harness('child', 'parent');
    fill();
    fireEvent.click(await screen.findByTestId('sign-toggle'));
    await unlock('parent-pass');
    expect(await screen.findByRole('alert')).toBeInTheDocument();
    expect(h.unlockOwnKey).not.toHaveBeenCalled();
  });

  it('refuses and locks a key the unlock reports for another account', async () => {
    const h = await harness();
    h.unlockOwnKey.mockImplementationOnce(async () => ({ keyRef: 'ref-x', accountId: 'someone-else' }));
    fill();
    fireEvent.click(await screen.findByTestId('sign-toggle'));
    await unlock('pw');
    await waitFor(() => expect(h.lockKeyRef).toHaveBeenCalledWith('ref-x'));
    fireEvent.click(screen.getByRole('button', { name: 'Send' }));
    expect(sends(h.client)).toBe(0);
    expect(screen.getByTestId('compose-sign-unlock')).toBeInTheDocument();
  });
});
