// Component tests for the key-management module (plan §3 e2 acceptance): own-key
// generation (calls the crypto-worker stub), armored import with a preview step,
// trust toggle, consent-gated WKD/VKS lookup rendering into the contact-key list,
// and per-contact key association — driven through the real keys store slice over
// the mock backend + the crypto-worker stub.

import { describe, it, expect, beforeEach } from 'vitest';
import { render, fireEvent, screen, waitFor, within } from '@solidjs/testing-library';
import { AppContext } from '../../state/context.ts';
import { createAppState, type AppState } from '../../state/store.ts';
import type { Client } from '../../api/client.ts';
import { __resetCryptoWorker } from '../../crypto/index.ts';
import { KeysModule } from './index.tsx';
import { makeKeysClient, defaultKeysSeed, ownPgpKey, contactPgpKey, cardsFrom, type KeysSeed } from './mockClient.ts';

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
    const card = await screen.findByRole('article', { name: 'Key me@example.org' });
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
    const card = await screen.findByRole('article', { name: 'Key me@example.org' });
    fireEvent.change(within(card).getByLabelText('Contact to associate'), { target: { value: 'c2' } });
    fireEvent.click(within(card).getByRole('button', { name: 'Associate' }));
    await waitFor(async () => {
      const cards = await cardsFrom(client);
      expect(cards.find((c) => c.id === 'c2')?.pgpKey).toContain('BEGIN PGP PUBLIC KEY');
    });
  });
});
