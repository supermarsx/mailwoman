import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@solidjs/testing-library';
import { createSignal } from 'solid-js';
import {
  CapabilityBanner,
  ComposeCrypto,
  DlpWarnings,
  EncryptSignToggles,
  type ComposeCryptoState,
} from './compose-crypto.tsx';
import {
  chooseRecipientKey,
  computeCapability,
  normalizeRecipients,
  type RecipientCapability,
} from './compose/capability.ts';
import { clearSignBody } from './compose/crypto-jmap.ts';
import { createStubCryptoWorker } from '../crypto/index.ts';
import type { CryptoKey, DlpVerdict } from '../api/crypto-types.ts';

// ── Fixtures ─────────────────────────────────────────────────────────────────

function pgpKey(over: Partial<CryptoKey> = {}): CryptoKey {
  return {
    id: 'k1',
    kind: 'pgp',
    isOwn: false,
    addresses: ['a@example.org'],
    fingerprint: 'FPR0000000000000000000000000000000000001',
    keyId: 'KEYID0000000001',
    algorithm: 'ed25519',
    createdAt: '2026-01-01T00:00:00Z',
    expiresAt: null,
    publicKeyArmored: '-----BEGIN PGP PUBLIC KEY BLOCK-----\nx\n-----END PGP PUBLIC KEY BLOCK-----',
    certPem: null,
    trust: 'tofu',
    autocrypt: true,
    source: 'harvested',
    hasPrivate: false,
    encryptedPrivateBackup: null,
    verifiedAt: null,
    keyHistory: [],
    ...over,
  };
}

function verdict(over: Partial<DlpVerdict> = {}): DlpVerdict {
  return {
    ruleId: 'r1',
    ruleName: 'No card numbers',
    action: 'warn',
    matchedDetectors: [],
    excerptRedacted: '',
    blocked: false,
    ...over,
  };
}

/** A lookup that returns a fixed set of keys per address. */
function lookupFrom(map: Record<string, CryptoKey[]>) {
  return vi.fn(async (address: string): Promise<CryptoKey[]> => map[address] ?? []);
}

// ── Pure capability logic ────────────────────────────────────────────────────

describe('capability logic', () => {
  const enc = (address: string): RecipientCapability => ({ address, encryptable: true, keyKind: 'pgp', publicKey: 'x' });
  const plain = (address: string): RecipientCapability => ({ address, encryptable: false, keyKind: null, publicKey: null });

  it('all recipients encryptable → e2ee', () => {
    expect(computeCapability([enc('a'), enc('b')])).toBe('e2ee');
  });
  it('no recipients encryptable → tls', () => {
    expect(computeCapability([plain('a'), plain('b')])).toBe('tls');
  });
  it('some but not all → mixed', () => {
    expect(computeCapability([enc('a'), plain('b')])).toBe('mixed');
  });
  it('no recipients at all → tls', () => {
    expect(computeCapability([])).toBe('tls');
  });

  it('chooseRecipientKey prefers a verified key and ignores revoked ones', () => {
    const cap = chooseRecipientKey('a@example.org', [
      pgpKey({ id: 'revoked', trust: 'revoked' }),
      pgpKey({ id: 'unver', trust: 'unverified', publicKeyArmored: 'UNVER' }),
      pgpKey({ id: 'ver', trust: 'verified', publicKeyArmored: 'VER' }),
    ]);
    expect(cap.encryptable).toBe(true);
    expect(cap.publicKey).toBe('VER');
  });
  it('chooseRecipientKey reports not-encryptable when only revoked keys exist', () => {
    const cap = chooseRecipientKey('a@example.org', [pgpKey({ trust: 'revoked' })]);
    expect(cap.encryptable).toBe(false);
    expect(cap.publicKey).toBeNull();
  });
  it('chooseRecipientKey uses the S/MIME cert when there is no PGP armor', () => {
    const cap = chooseRecipientKey('a@example.org', [
      pgpKey({ kind: 'smime', publicKeyArmored: null, certPem: '-----BEGIN CERTIFICATE-----' }),
    ]);
    expect(cap.keyKind).toBe('smime');
    expect(cap.publicKey).toContain('CERTIFICATE');
  });

  it('normalizeRecipients trims, lowercases, dedupes and drops blanks', () => {
    expect(normalizeRecipients([' A@X.org ', 'a@x.org', '', 'b@x.org'])).toEqual(['a@x.org', 'b@x.org']);
  });
});

// ── Presentational subcomponents ─────────────────────────────────────────────

describe('CapabilityBanner', () => {
  it('renders each capability with an accessible live-region status role', () => {
    const { unmount } = render(() => <CapabilityBanner capability="e2ee" recipients={[]} />);
    const banner = screen.getByTestId('compose-crypto-banner');
    expect(banner).toHaveAttribute('data-capability', 'e2ee');
    expect(banner).toHaveAttribute('role', 'status');
    expect(banner).toHaveAttribute('aria-live', 'polite');
    expect(banner.textContent).toContain('End-to-end encrypted');
    unmount();
  });

  it('lists the TLS-only recipients when mixed', () => {
    render(() => (
      <CapabilityBanner
        capability="mixed"
        recipients={[
          { address: 'has@x.org', encryptable: true, keyKind: 'pgp', publicKey: 'x' },
          { address: 'none@x.org', encryptable: false, keyKind: null, publicKey: null },
        ]}
      />
    ));
    const banner = screen.getByTestId('compose-crypto-banner');
    expect(banner.textContent).toContain('none@x.org');
    expect(banner.textContent).not.toContain('has@x.org');
  });
});

describe('EncryptSignToggles', () => {
  it('exposes encrypt/sign checkboxes and fires their callbacks', () => {
    const onEncrypt = vi.fn();
    const onSign = vi.fn();
    render(() => (
      <EncryptSignToggles
        encrypt={false}
        sign={false}
        protectSubject={false}
        onEncryptChange={onEncrypt}
        onSignChange={onSign}
        onProtectSubjectChange={() => undefined}
      />
    ));
    fireEvent.click(screen.getByTestId('encrypt-toggle'));
    fireEvent.click(screen.getByTestId('sign-toggle'));
    expect(onEncrypt).toHaveBeenCalledWith(true);
    expect(onSign).toHaveBeenCalledWith(true);
  });

  it('disables the encrypt switch with a reason when no key is available', () => {
    render(() => (
      <EncryptSignToggles
        encrypt={false}
        sign={false}
        protectSubject={false}
        encryptDisabled
        encryptDisabledReason="No recipient encryption key available."
        onEncryptChange={() => undefined}
        onSignChange={() => undefined}
        onProtectSubjectChange={() => undefined}
      />
    ));
    expect(screen.getByTestId('encrypt-toggle')).toBeDisabled();
    expect(screen.getByText('No recipient encryption key available.')).toBeInTheDocument();
  });

  it('shows the protected-subject affordance only while encryption is on', () => {
    const [encrypt, setEncrypt] = createSignal(false);
    render(() => (
      <EncryptSignToggles
        encrypt={encrypt()}
        sign={false}
        protectSubject={false}
        onEncryptChange={() => undefined}
        onSignChange={() => undefined}
        onProtectSubjectChange={() => undefined}
      />
    ));
    expect(screen.queryByTestId('protect-subject-toggle')).toBeNull();
    setEncrypt(true);
    expect(screen.getByTestId('protect-subject-toggle')).toBeInTheDocument();
  });
});

describe('DlpWarnings', () => {
  it('renders nothing when there are no verdicts', () => {
    render(() => <DlpWarnings verdicts={[]} />);
    expect(screen.queryByTestId('dlp-warnings')).toBeNull();
  });

  it('surfaces a warn verdict politely with the rule name', () => {
    render(() => <DlpWarnings verdicts={[verdict({ action: 'warn', ruleName: 'External domain' })]} />);
    const list = screen.getByTestId('dlp-warnings');
    expect(list).toHaveAttribute('role', 'status');
    expect(screen.getByTestId('dlp-warn').textContent).toContain('External domain');
  });

  it('surfaces a block verdict assertively with the rule message', () => {
    render(() => (
      <DlpWarnings
        verdicts={[verdict({ action: 'block', ruleName: 'No card numbers', excerptRedacted: '•••• 1111', matchedDetectors: ['pan'] })]}
      />
    ));
    const list = screen.getByTestId('dlp-warnings');
    expect(list).toHaveAttribute('role', 'alert');
    const block = screen.getByTestId('dlp-block');
    expect(block).toHaveAttribute('data-action', 'block');
    expect(block.textContent).toContain('No card numbers');
    expect(block.textContent).toContain('•••• 1111');
    expect(block.textContent).toContain('pan');
  });
});

// ── Container: live banner + DLP + worker wiring ─────────────────────────────

describe('ComposeCrypto (container)', () => {
  const noDlp = vi.fn(async (): Promise<DlpVerdict[]> => []);

  it('banner reads E2EE when every recipient has a key', async () => {
    const [recipients] = createSignal(['a@example.org', 'b@example.org']);
    render(() => (
      <ComposeCrypto
        recipients={recipients}
        lookupKeys={lookupFrom({ 'a@example.org': [pgpKey()], 'b@example.org': [pgpKey()] })}
        scanDlp={noDlp}
      />
    ));
    await waitFor(() =>
      expect(screen.getByTestId('compose-crypto-banner')).toHaveAttribute('data-capability', 'e2ee'),
    );
  });

  it('banner reads TLS when no recipient has a key', async () => {
    const [recipients] = createSignal(['a@example.org']);
    render(() => (
      <ComposeCrypto recipients={recipients} lookupKeys={lookupFrom({})} scanDlp={noDlp} />
    ));
    await waitFor(() =>
      expect(screen.getByTestId('compose-crypto-banner')).toHaveAttribute('data-capability', 'tls'),
    );
    // ...and the encrypt switch is disabled since nothing can be encrypted.
    expect(screen.getByTestId('encrypt-toggle')).toBeDisabled();
  });

  it('banner reads mixed when only some recipients have a key', async () => {
    const [recipients] = createSignal(['a@example.org', 'b@example.org']);
    render(() => (
      <ComposeCrypto
        recipients={recipients}
        lookupKeys={lookupFrom({ 'a@example.org': [pgpKey()] })}
        scanDlp={noDlp}
      />
    ));
    await waitFor(() =>
      expect(screen.getByTestId('compose-crypto-banner')).toHaveAttribute('data-capability', 'mixed'),
    );
  });

  it('updates the banner live as recipients change', async () => {
    const [recipients, setRecipients] = createSignal(['a@example.org']);
    render(() => (
      <ComposeCrypto
        recipients={recipients}
        lookupKeys={lookupFrom({ 'a@example.org': [pgpKey()], 'b@example.org': [] })}
        scanDlp={noDlp}
      />
    ));
    await waitFor(() =>
      expect(screen.getByTestId('compose-crypto-banner')).toHaveAttribute('data-capability', 'e2ee'),
    );
    setRecipients(['a@example.org', 'b@example.org']);
    await waitFor(() =>
      expect(screen.getByTestId('compose-crypto-banner')).toHaveAttribute('data-capability', 'mixed'),
    );
  });

  it('enabling encryption calls the crypto worker to encrypt the draft', async () => {
    const [recipients] = createSignal(['a@example.org']);
    const [body] = createSignal('secret plans');
    const worker = createStubCryptoWorker();
    const encryptSpy = vi.spyOn(worker, 'encrypt');
    render(() => (
      <ComposeCrypto
        recipients={recipients}
        bodyText={body}
        lookupKeys={lookupFrom({ 'a@example.org': [pgpKey({ publicKeyArmored: 'RECIPIENT_KEY' })] })}
        scanDlp={noDlp}
        cryptoWorker={worker}
      />
    ));
    const toggle = await screen.findByTestId('encrypt-toggle');
    await waitFor(() => expect(toggle).not.toBeDisabled());

    fireEvent.click(toggle);

    await waitFor(() => expect(encryptSpy).toHaveBeenCalledTimes(1));
    expect(encryptSpy.mock.calls[0]![0]).toMatchObject({
      kind: 'pgp',
      plaintext: 'secret plans',
      recipientPublicKeys: ['RECIPIENT_KEY'],
    });
    // The encrypted-draft affordance appears once the worker resolves.
    await waitFor(() => expect(screen.getByTestId('encrypted-draft-indicator')).toBeInTheDocument());
  });

  it('re-encrypts with the subject when protected-subject is turned on', async () => {
    const [recipients] = createSignal(['a@example.org']);
    const [subject] = createSignal('Q3 numbers');
    const [body] = createSignal('secret');
    const worker = createStubCryptoWorker();
    const encryptSpy = vi.spyOn(worker, 'encrypt');
    render(() => (
      <ComposeCrypto
        recipients={recipients}
        subject={subject}
        bodyText={body}
        lookupKeys={lookupFrom({ 'a@example.org': [pgpKey()] })}
        scanDlp={noDlp}
        cryptoWorker={worker}
      />
    ));
    const toggle = await screen.findByTestId('encrypt-toggle');
    await waitFor(() => expect(toggle).not.toBeDisabled());
    fireEvent.click(toggle);
    await waitFor(() => expect(encryptSpy).toHaveBeenCalledTimes(1));

    fireEvent.click(screen.getByTestId('protect-subject-toggle'));
    await waitFor(() => expect(encryptSpy).toHaveBeenCalledTimes(2));
    expect(encryptSpy.mock.calls[1]![0]).toMatchObject({ protectedSubject: 'Q3 numbers' });
  });

  it('reports DLP verdicts and clears canSend on a block', async () => {
    const [recipients] = createSignal(['a@example.org']);
    const [body] = createSignal('card 4111 1111 1111 1111');
    const states: ComposeCryptoState[] = [];
    render(() => (
      <ComposeCrypto
        recipients={recipients}
        bodyText={body}
        lookupKeys={lookupFrom({})}
        scanDlp={vi.fn(async (): Promise<DlpVerdict[]> => [
          verdict({ action: 'block', ruleName: 'No card numbers', blocked: true, matchedDetectors: ['pan'] }),
        ])}
        onChange={(s) => states.push(s)}
      />
    ));
    await waitFor(() => expect(screen.getByTestId('dlp-block')).toBeInTheDocument());
    expect(screen.getByTestId('dlp-block').textContent).toContain('No card numbers');
    await waitFor(() => expect(states.at(-1)?.canSend).toBe(false));
  });

  it('keeps canSend true for a warn-only verdict', async () => {
    const [recipients] = createSignal(['a@example.org']);
    const [body] = createSignal('hello');
    const states: ComposeCryptoState[] = [];
    render(() => (
      <ComposeCrypto
        recipients={recipients}
        bodyText={body}
        lookupKeys={lookupFrom({})}
        scanDlp={vi.fn(async (): Promise<DlpVerdict[]> => [verdict({ action: 'warn' })])}
        onChange={(s) => states.push(s)}
      />
    ));
    await waitFor(() => expect(screen.getByTestId('dlp-warn')).toBeInTheDocument());
    expect(states.at(-1)?.canSend).toBe(true);
  });
});

// ── Sign-on-send: encrypt+sign folds a signature via signWithKeyRef ──────────

describe('ComposeCrypto — sign-on-send', () => {
  const noDlp = vi.fn(async (): Promise<DlpVerdict[]> => []);

  it('folds a signature into the encrypt call via signWithKeyRef when sign is on', async () => {
    const [recipients] = createSignal(['a@example.org']);
    const [body] = createSignal('secret');
    const worker = createStubCryptoWorker();
    const encryptSpy = vi.spyOn(worker, 'encrypt');
    render(() => (
      <ComposeCrypto
        recipients={recipients}
        bodyText={body}
        lookupKeys={lookupFrom({ 'a@example.org': [pgpKey({ publicKeyArmored: 'RCPT' })] })}
        scanDlp={noDlp}
        cryptoWorker={worker}
        signingKeyRef={() => 'keyref-1'}
      />
    ));
    const encToggle = await screen.findByTestId('encrypt-toggle');
    await waitFor(() => expect(encToggle).not.toBeDisabled());

    // Encrypt with sign still off → no signature folded in.
    fireEvent.click(encToggle);
    await waitFor(() => expect(encryptSpy).toHaveBeenCalledTimes(1));
    expect(encryptSpy.mock.calls[0]![0].signWithKeyRef).toBeUndefined();

    // Turn sign on with the key already unlocked → re-encrypt WITH signWithKeyRef.
    fireEvent.click(screen.getByTestId('sign-toggle'));
    await waitFor(() => expect(encryptSpy).toHaveBeenCalledTimes(2));
    expect(encryptSpy.mock.calls[1]![0]).toMatchObject({ signWithKeyRef: 'keyref-1' });
  });

  it('asks the host to unlock the signing key when sign is enabled while locked', async () => {
    const [recipients] = createSignal(['a@example.org']);
    const onRequestSigningKey = vi.fn();
    render(() => (
      <ComposeCrypto
        recipients={recipients}
        lookupKeys={lookupFrom({})}
        scanDlp={noDlp}
        signingKeyRef={() => null}
        onRequestSigningKey={onRequestSigningKey}
      />
    ));
    fireEvent.click(await screen.findByTestId('sign-toggle'));
    expect(onRequestSigningKey).toHaveBeenCalledTimes(1);
  });

  it('re-encrypts to add the signature once the key is unlocked after toggling sign', async () => {
    const [recipients] = createSignal(['a@example.org']);
    const [body] = createSignal('secret');
    const [ref, setRef] = createSignal<string | null>(null);
    const worker = createStubCryptoWorker();
    const encryptSpy = vi.spyOn(worker, 'encrypt');
    render(() => (
      <ComposeCrypto
        recipients={recipients}
        bodyText={body}
        lookupKeys={lookupFrom({ 'a@example.org': [pgpKey()] })}
        scanDlp={noDlp}
        cryptoWorker={worker}
        signingKeyRef={ref}
        onRequestSigningKey={() => undefined}
      />
    ));
    const encToggle = await screen.findByTestId('encrypt-toggle');
    await waitFor(() => expect(encToggle).not.toBeDisabled());
    fireEvent.click(encToggle);
    await waitFor(() => expect(encryptSpy).toHaveBeenCalled());

    // Sign on while still locked → encrypt runs but without a keyRef.
    fireEvent.click(screen.getByTestId('sign-toggle'));
    await waitFor(() => expect(encryptSpy.mock.calls.at(-1)![0].signWithKeyRef).toBeUndefined());
    const before = encryptSpy.mock.calls.length;

    // Unlock completes → the deferred effect re-encrypts WITH the signature.
    setRef('keyref-late');
    await waitFor(() => expect(encryptSpy.mock.calls.length).toBeGreaterThan(before));
    expect(encryptSpy.mock.calls.at(-1)![0]).toMatchObject({ signWithKeyRef: 'keyref-late' });
  });
});

// ── Sign-only: clear-sign an unencrypted body ────────────────────────────────

describe('clearSignBody (sign-only clear-signed send)', () => {
  it('clear-signs the body via the worker sign() with detached:false', async () => {
    const worker = createStubCryptoWorker();
    const signSpy = vi.spyOn(worker, 'sign');
    const out = await clearSignBody(worker, { keyRef: 'r', bundle: 'BUNDLE', passphrase: 'pw' }, 'hello world');
    expect(signSpy).toHaveBeenCalledTimes(1);
    expect(signSpy.mock.calls[0]![0]).toMatchObject({
      kind: 'pgp',
      data: 'hello world',
      encryptedPrivateBundle: 'BUNDLE',
      passphrase: 'pw',
      detached: false,
    });
    expect(out).toContain('PGP SIGNATURE');
  });
});
