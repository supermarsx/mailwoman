// Component tests for the key-management module (plan §3 e2 acceptance): own-key
// generation (calls the crypto-worker stub), armored import with a preview step,
// trust toggle, consent-gated WKD/VKS lookup rendering into the contact-key list,
// and per-contact key association — driven through the real keys store slice over
// the mock backend + the crypto-worker stub.

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { AppContext } from '../../state/context.ts';
import { createAppState, type AppState } from '../../state/store.ts';
import type { Client } from '../../api/client.ts';
import { __resetCryptoWorker, getCryptoWorker } from '../../crypto/index.ts';
import { t } from '../../test/i18n.ts';
import { isolate } from '../../i18n/index.ts';
import { KeysModule } from './index.tsx';
import { makeKeysClient, defaultKeysSeed, ownPgpKey, contactPgpKey, cardsFrom, type KeysSeed } from './mockClient.ts';

const SMIME_CERT = '-----BEGIN CERTIFICATE-----\n(mock)\n-----END CERTIFICATE-----';

/** An own S/MIME key as the server returns it after a reload (bundle in the opaque backup). */
function ownSmimeKey(): ReturnType<typeof ownPgpKey> {
  return ownPgpKey({
    id: 'key-smime-1',
    kind: 'smime',
    addresses: ['smime@example.org'],
    fingerprint: 'AB12'.repeat(16),
    keyId: 'AB12'.repeat(4),
    algorithm: 'rsa-3072',
    expiresAt: '2028-10-04T00:00:00Z',
    publicKeyArmored: null,
    certPem: SMIME_CERT,
    autocrypt: false,
    source: 'generated',
    encryptedPrivateBackup: 'WRAPPED-SMIME-KEY',
  });
}

function renderModule(seed: KeysSeed = defaultKeysSeed()): { app: AppState; client: Client } {
  const client = makeKeysClient(seed);
  const app = createAppState(client);
  render(() => (
    <AppContext.Provider value={app}>
      <KeysModule />
    </AppContext.Provider>
  ));
  return { app, client };
}

describe('KeysModule', () => {
  beforeEach(() => {
    localStorage.clear();
    __resetCryptoWorker();
    vi.restoreAllMocks();
  });

  it('lists the account own keys after load', async () => {
    renderModule();
    const ownList = await screen.findByRole('list', { name: 'Your keys' });
    expect(within(ownList).getByText('me@example.org')).toBeInTheDocument();
  });

  it('renders both own and contact keys in their sections', async () => {
    renderModule({ keys: [ownPgpKey(), contactPgpKey()], contacts: [] });
    const own = await screen.findByRole('list', { name: 'Your keys' });
    const contact = await screen.findByRole('list', { name: 'Contact keys' });
    expect(within(own).getByText('me@example.org')).toBeInTheDocument();
    expect(within(contact).getByText('alan@example.org')).toBeInTheDocument();
  });

  it('shows fingerprint safe words and a QR when a key is selected', async () => {
    renderModule();
    fireEvent.click(await screen.findByRole('button', { name: /me@example\.org/ }));
    const card = await screen.findByRole('article', { name: t('keys-key-card', { name: isolate('me@example.org') }) });
    const words = within(card).getByRole('list', { name: 'Fingerprint safe words' });
    expect(words.querySelectorAll('li')).toHaveLength(10); // 160-bit fingerprint → 10 proquints
    expect(within(card).getByRole('img', { name: 'Fingerprint QR code' })).toBeInTheDocument();
  });

  it('generates a new own key through the worker stub and lists it', async () => {
    const { app } = renderModule();
    await screen.findByText('me@example.org');
    const before = app.ownKeys().length;
    fireEvent.click(screen.getByRole('button', { name: 'Generate key' }));
    const dialog = await screen.findByRole('dialog', { name: 'Generate a key' });
    fireEvent.input(within(dialog).getByLabelText('Email'), { target: { value: 'alice@example.org' } });
    fireEvent.input(within(dialog).getByLabelText('Key passphrase'), { target: { value: 'hunter2' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Generate' }));
    await waitFor(() => expect(app.ownKeys().length).toBe(before + 1));
    const ownList = await screen.findByRole('list', { name: 'Your keys' });
    expect(within(ownList).getByText('alice@example.org')).toBeInTheDocument();
  });

  // Audit §13 row 35: S/MIME generation used to produce an OpenPGP key filed as
  // a certificate, and was removed. It is back with a real certificate behind it;
  // these cases pin what the dialog offers and what each choice stores.
  it('offers OpenPGP and S/MIME, and says what an S/MIME key is only while it is chosen', async () => {
    renderModule();
    await screen.findByText('me@example.org');
    fireEvent.click(screen.getByRole('button', { name: 'Generate key' }));
    const dialog = await screen.findByRole('dialog', { name: 'Generate a key' });
    const type = within(dialog).getByLabelText('Key type') as HTMLSelectElement;
    expect(Array.from(type.options).map((o) => o.value)).toEqual(['pgp', 'smime']);
    expect(type.value).toBe('pgp');
    expect(within(dialog).queryByText(/self-signed/)).toBeNull();

    fireEvent.change(type, { target: { value: 'smime' } });
    const note = within(dialog).getByText(/self-signed/);
    // What is made, that others will not trust it as is, and what the app cannot do with it yet.
    expect(note.textContent).toMatch(/RSA 3072-bit key and a certificate/);
    expect(note.textContent).toMatch(/will not trust it/);
    expect(note.textContent).toMatch(/does not yet sign or decrypt mail with it/);

    fireEvent.change(type, { target: { value: 'pgp' } });
    expect(within(dialog).queryByText(/self-signed/)).toBeNull();
  });

  it('generates an OpenPGP key when the type is left alone', async () => {
    const { app } = renderModule();
    await screen.findByText('me@example.org');
    const before = app.ownKeys().length;
    fireEvent.click(screen.getByRole('button', { name: 'Generate key' }));
    const dialog = await screen.findByRole('dialog', { name: 'Generate a key' });
    fireEvent.input(within(dialog).getByLabelText('Email'), { target: { value: 'carol@example.org' } });
    fireEvent.input(within(dialog).getByLabelText('Key passphrase'), { target: { value: 'hunter2' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Generate' }));
    await waitFor(() => expect(app.ownKeys().length).toBe(before + 1));
    const made = app.ownKeys().find((k) => k.addresses.includes('carol@example.org'));
    expect(made?.kind).toBe('pgp');
    expect(made?.certPem).toBeNull();
    expect(made?.publicKeyArmored).toContain('BEGIN PGP PUBLIC KEY BLOCK');
  });

  it('generates an S/MIME key with a certificate when S/MIME is chosen', async () => {
    const { app } = renderModule();
    await screen.findByText('me@example.org');
    const worker = vi.spyOn(getCryptoWorker(), 'generateKey');
    fireEvent.click(screen.getByRole('button', { name: 'Generate key' }));
    const dialog = await screen.findByRole('dialog', { name: 'Generate a key' });
    fireEvent.change(within(dialog).getByLabelText('Key type'), { target: { value: 'smime' } });
    fireEvent.input(within(dialog).getByLabelText('Name'), { target: { value: 'Carol' } });
    fireEvent.input(within(dialog).getByLabelText('Email'), { target: { value: 'carol@example.org' } });
    fireEvent.input(within(dialog).getByLabelText('Key passphrase'), { target: { value: 'hunter2' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Generate' }));
    await waitFor(() => expect(app.ownKeys().some((k) => k.kind === 'smime')).toBe(true));

    expect(worker).toHaveBeenCalledWith({ kind: 'smime', userId: 'Carol <carol@example.org>', passphrase: 'hunter2' });
    const made = app.ownKeys().find((k) => k.kind === 'smime')!;
    expect(made.certPem).toContain('BEGIN CERTIFICATE');
    expect(made.publicKeyArmored).toBeNull();
    expect(made.algorithm).toBe('rsa-3072');
    expect(made.source).toBe('generated');
    // The dialog closed onto the new key's detail, which says what the certificate is.
    const section = await screen.findByRole('group', { name: 'S/MIME certificate' });
    expect(within(section).getByText(/This certificate is self-signed/)).toBeInTheDocument();
    expect(within(section).getByText(/does not yet sign or decrypt mail with an S\/MIME key/)).toBeInTheDocument();
  });

  it('says so in the dialog when generation fails, and stores nothing', async () => {
    const { app } = renderModule();
    await screen.findByText('me@example.org');
    vi.spyOn(getCryptoWorker(), 'generateKey').mockRejectedValue(new Error('the email address is not one a certificate can carry'));
    const before = app.ownKeys().length;
    fireEvent.click(screen.getByRole('button', { name: 'Generate key' }));
    const dialog = await screen.findByRole('dialog', { name: 'Generate a key' });
    fireEvent.change(within(dialog).getByLabelText('Key type'), { target: { value: 'smime' } });
    fireEvent.input(within(dialog).getByLabelText('Email'), { target: { value: 'ünï@example.org' } });
    fireEvent.input(within(dialog).getByLabelText('Key passphrase'), { target: { value: 'hunter2' } });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Generate' }));
    const alert = await within(dialog).findByRole('alert');
    expect(alert.textContent).toMatch(/No key was generated/);
    expect(app.ownKeys().length).toBe(before);
    expect(screen.getByRole('dialog', { name: 'Generate a key' })).toBeInTheDocument();
  });

  it('offers PKCS#12 export, a certificate request and issued-certificate import for an own S/MIME key', async () => {
    const createObjectURL = vi.fn((_blob: Blob) => 'blob:x');
    Object.assign(URL, { createObjectURL, revokeObjectURL: vi.fn() });
    const click = vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(() => undefined);
    const { app } = renderModule({ keys: [ownPgpKey(), ownSmimeKey()], contacts: [] });
    const exportP12 = vi.spyOn(getCryptoWorker(), 'exportPkcs12');
    const request = vi.spyOn(getCryptoWorker(), 'certificateRequest');

    fireEvent.click(await screen.findByRole('button', { name: /smime@example\.org/ }));
    const section = await screen.findByRole('group', { name: 'S/MIME certificate' });
    expect(within(section).getByText('Valid until 2028-10-04.')).toBeInTheDocument();
    // An Autocrypt Setup Message is OpenPGP: not offered for this key.
    expect(screen.queryByRole('button', { name: 'Export backup' })).toBeNull();

    const exportButton = within(section).getByRole('button', { name: 'Export as PKCS#12 (.p12)' });
    const requestButton = within(section).getByRole('button', { name: 'Download certificate request (.csr)' });
    const attachButton = within(section).getByRole('button', { name: 'Import issued certificate' });
    // Nothing runs without the passphrase.
    expect(exportButton).toBeDisabled();
    expect(requestButton).toBeDisabled();
    expect(attachButton).toBeDisabled();

    fireEvent.input(within(section).getByLabelText('Passphrase for this S/MIME key'), { target: { value: 'hunter2' } });
    expect(attachButton).toBeDisabled(); // still no file chosen

    fireEvent.click(exportButton);
    await waitFor(() => expect(click).toHaveBeenCalledTimes(1));
    expect(exportP12).toHaveBeenCalledWith({
      certPem: SMIME_CERT,
      encryptedPrivateBundle: 'WRAPPED-SMIME-KEY',
      passphrase: 'hunter2',
    });
    expect(createObjectURL.mock.calls[0]![0].type).toBe('application/x-pkcs12');

    fireEvent.click(requestButton);
    await waitFor(() => expect(click).toHaveBeenCalledTimes(2));
    expect(request).toHaveBeenCalledTimes(1);
    expect(createObjectURL.mock.calls[1]![0].type).toBe('application/pkcs10');

    // A refused certificate is reported in place and the key is left as it was.
    vi.spyOn(getCryptoWorker(), 'attachIssuedCert').mockRejectedValue(new Error('the certificate is not for this private key'));
    // jsdom's File has no arrayBuffer(); the component reads the file through it.
    const file = Object.assign(new File([new Uint8Array([0x30, 0x00])], 'other.crt'), {
      arrayBuffer: async () => new Uint8Array([0x30, 0x00]).buffer,
    });
    fireEvent.change(within(section).getByLabelText('Issued certificate file'), { target: { files: [file] } });
    await waitFor(() => expect(attachButton).toBeEnabled());
    fireEvent.click(attachButton);
    const alert = await within(section).findByRole('alert');
    expect(alert.textContent).toMatch(/The certificate was not imported/);
    expect(app.ownKeys().find((k) => k.kind === 'smime')?.certPem).toBe(SMIME_CERT);
  });

  it('shows no S/MIME certificate section for an OpenPGP key', async () => {
    renderModule();
    fireEvent.click(await screen.findByRole('button', { name: /me@example\.org/ }));
    await screen.findByRole('article', { name: t('keys-key-card', { name: isolate('me@example.org') }) });
    expect(screen.queryByRole('group', { name: 'S/MIME certificate' })).toBeNull();
  });

  it('previews an armored import before committing it', async () => {
    const { app } = renderModule();
    await screen.findByText('me@example.org');
    fireEvent.click(screen.getByRole('button', { name: 'Import key' }));
    const dialog = await screen.findByRole('dialog', { name: 'Import a key' });
    fireEvent.input(within(dialog).getByLabelText('Armored key'), {
      target: { value: '-----BEGIN PGP PUBLIC KEY BLOCK-----\nx\n-----END PGP PUBLIC KEY BLOCK-----' },
    });
    fireEvent.click(within(dialog).getByRole('button', { name: 'Preview' }));
    // The preview surfaces the parsed key BEFORE anything is persisted.
    const preview = await within(dialog).findByRole('group', { name: 'Import preview' });
    expect(within(preview).getByText(/Fingerprint:/)).toBeInTheDocument();
    const before = app.keys().length;
    fireEvent.click(within(dialog).getByRole('button', { name: 'Import' }));
    await waitFor(() => expect(app.keys().length).toBe(before + 1));
  });

  it('toggles a key trust level through CryptoKey/setTrust', async () => {
    const { app } = renderModule();
    fireEvent.click(await screen.findByRole('button', { name: /me@example\.org/ }));
    const select = (await screen.findByLabelText('Trust level')) as HTMLSelectElement;
    expect(select.value).toBe('verified');
    fireEvent.change(select, { target: { value: 'revoked' } });
    await waitFor(() => expect(app.keys().find((k) => k.id === 'key-pgp-1')?.trust).toBe('revoked'));
  });

  it('looks up a key with consent and renders it in the contact-key list', async () => {
    renderModule();
    await screen.findByText('me@example.org');
    fireEvent.input(screen.getByLabelText('Address to look up'), { target: { value: 'alan@example.org' } });
    // The Look up button is gated on the consent checkbox.
    const lookup = screen.getByRole('button', { name: 'Look up' });
    expect(lookup).toBeDisabled();
    fireEvent.click(screen.getByLabelText('Consent to external lookup'));
    expect(lookup).toBeEnabled();
    fireEvent.click(lookup);
    const contactList = await screen.findByRole('list', { name: 'Contact keys' });
    expect(await within(contactList).findByText('alan@example.org')).toBeInTheDocument();
  });

  it('associates a key with a contact, writing ContactCard.pgpKey', async () => {
    const { app, client } = renderModule();
    // Wait for contacts to load so the association picker is populated.
    await waitFor(() => expect(app.contacts().length).toBeGreaterThan(0));
    fireEvent.click(await screen.findByRole('button', { name: /me@example\.org/ }));
    const card = await screen.findByRole('article', { name: t('keys-key-card', { name: isolate('me@example.org') }) });
    fireEvent.change(within(card).getByLabelText('Contact to associate'), { target: { value: 'c2' } });
    fireEvent.click(within(card).getByRole('button', { name: 'Associate' }));
    await waitFor(async () => {
      const cards = await cardsFrom(client);
      expect(cards.find((c) => c.id === 'c2')?.pgpKey).toContain('BEGIN PGP PUBLIC KEY');
    });
  });
});
