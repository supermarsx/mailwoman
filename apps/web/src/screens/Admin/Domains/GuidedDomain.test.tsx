import { expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { GuidedDomain } from './GuidedDomain.tsx';
import { domainsApi, type DomainStatus } from '../../../api/admin/domains.ts';
import { renderInShell } from '../shell/testkit.tsx';
import { anyDirty } from '../shell/useDirtyGuard.ts';

const status = (mode: 'engine' | 'proxy'): DomainStatus => ({ mode, upstreamChoices: [], blockedAccounts: { total: 0, byDomain: [] } });
const next = () => fireEvent.click(screen.getByRole('button', { name: 'Next' }));
async function identity() {
  fireEvent.input(await screen.findByLabelText('Domain name'), { target: { value: 'Example.ORG' } });
  next();
}
function setup(mode: 'engine' | 'proxy' = 'engine', canUpstream = true) {
  const save = vi.fn<typeof domainsApi.save>(async () => ({ name: 'example.org', enabled: true, preferred: false, upstream: {}, settings: {}, allowlist: [], blocklist: [], updatedAt: 0 }));
  const created = vi.fn(async () => { expect(anyDirty()).toBe(false); });
  renderInShell(() => <GuidedDomain api={{ ...domainsApi, save }} status={status(mode)} canUpstream={canUpstream} onCreated={created} onCancel={() => {}} />);
  return { save, created };
}
it('creates only from review, normalizes domain, and clears dirty state before navigation', async () => {
  const { save, created } = setup();
  await identity();
  expect(anyDirty()).toBe(true);
  next();
  expect(save).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole('button', { name: 'Back' }));
  next();
  fireEvent.click(screen.getByRole('button', { name: 'Create domain' }));
  await waitFor(() => expect(created).toHaveBeenCalledWith('example.org'));
  expect(save).toHaveBeenCalledExactlyOnceWith('example.org', { enabled: true }, null);
});
it('never emits upstream fields for administrators without upstream permission', async () => {
  const { save } = setup('engine', false);
  await identity();
  expect(screen.queryByRole('combobox')).not.toBeInTheDocument();
  next();
  fireEvent.click(screen.getByRole('button', { name: 'Create domain' }));
  await waitFor(() => expect(save).toHaveBeenCalledExactlyOnceWith('example.org', { enabled: true }, null));
});
it('does not recreate a saved domain if the post-save navigation fails', async () => {
  const { save, created } = setup();
  created.mockRejectedValueOnce(new Error('synthetic navigation failure'));
  await identity(); next();
  fireEvent.click(screen.getByRole('button', { name: 'Create domain' }));
  await waitFor(() => expect(created).toHaveBeenCalledTimes(1));
  await waitFor(() => expect(screen.getByRole('button', { name: 'Create domain' })).not.toBeDisabled());
  fireEvent.click(screen.getByRole('button', { name: 'Create domain' }));
  await waitFor(() => expect(created).toHaveBeenCalledTimes(2));
  expect(save).toHaveBeenCalledTimes(1);
});
it('validates endpoints and saves POP3 with optional verified SMTP only at review', async () => {
  const { save } = setup();
  await identity();
  fireEvent.change(screen.getByLabelText('Connection'), { target: { value: 'manual' } });
  next();
  expect(screen.getByRole('alert')).toHaveTextContent('ports between 1 and 65535');
  fireEvent.change(screen.getByLabelText('Mail protocol'), { target: { value: 'pop3' } });
  fireEvent.input(screen.getByLabelText('Mail hostname or IP address'), { target: { value: 'mail.example.org' } });
  fireEvent.input(screen.getByLabelText('Mail port'), { target: { value: '65536' } });
  next();
  expect(save).not.toHaveBeenCalled();
  fireEvent.input(screen.getByLabelText('Mail port'), { target: { value: '995' } });
  fireEvent.click(screen.getByLabelText('Configure an outgoing SMTP server'));
  fireEvent.input(screen.getByLabelText('SMTP hostname or IP address'), { target: { value: 'smtp.example.org' } });
  next();
  expect(save).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole('button', { name: 'Create domain' }));
  await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
  expect(save.mock.calls[0]?.[1]).toEqual({ enabled: true, upstream: { pop3: { host: 'mail.example.org', port: 995, tls: 'implicit', auth: 'password', allowPrivate: false }, smtp: { host: 'smtp.example.org', port: 465, tls: 'implicit', auth: 'password', allowPrivate: false } } });
});
it('rejects domain paths and unsafe JMAP URLs, retaining review after a save failure', async () => {
  const { save } = setup('proxy');
  fireEvent.input(await screen.findByLabelText('Domain name'), { target: { value: 'example.org/path' } });
  next();
  expect(screen.getByRole('alert')).toHaveTextContent('without a scheme, path or port');
  fireEvent.input(screen.getByLabelText('Domain name'), { target: { value: 'example.org' } });
  next();
  fireEvent.change(screen.getByLabelText('Connection'), { target: { value: 'manual' } });
  for (const value of ['http://mail.example.org/jmap', 'https://secret@mail.example.org/jmap', 'https://mail.example.org/jmap?token=secret']) {
    fireEvent.input(screen.getByLabelText('JMAP session URL'), { target: { value } }); next();
    expect(screen.getByRole('alert')).toHaveTextContent('HTTPS JMAP');
  }
  fireEvent.input(screen.getByLabelText('JMAP session URL'), { target: { value: 'https://mail.example.org/jmap' } });
  next();
  save.mockRejectedValueOnce(new Error('synthetic save failure'));
  fireEvent.click(screen.getByRole('button', { name: 'Create domain' }));
  await waitFor(() => expect(screen.getByRole('button', { name: 'Create domain' })).not.toBeDisabled());
  expect(anyDirty()).toBe(true);
  expect(document.querySelector('[data-step="review"]')).toBeInTheDocument();
});
