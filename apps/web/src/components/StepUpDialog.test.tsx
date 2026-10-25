import { expect, it, vi } from 'vitest';
import { fireEvent, screen, waitFor } from '@solidjs/testing-library';
import { requestStepUp } from './StepUpDialog.tsx';
import { assertPasskey } from '../screens/Settings/webauthn.ts';
vi.mock('../screens/Settings/webauthn.ts', () => ({ assertPasskey: vi.fn() }));
it('submits password and code then removes the secret inputs', async () => {
  const send = vi.fn(async () => new Response('{}', { status: 200 }));
  const result = requestStepUp({ methods: ['password', 'totp'], send });
  const dialog = await screen.findByRole('dialog', { name: 'Confirm it is you' });
  expect(dialog).toHaveAttribute('aria-modal', 'true');
  fireEvent.input(screen.getByLabelText('Password'), { target: { value: 'private-password' } });
  fireEvent.input(screen.getByLabelText('Authenticator code'), { target: { value: '123456' } });
  fireEvent.submit(dialog);
  await expect(result).resolves.toBe(true);
  expect(send).toHaveBeenCalledWith({ password: 'private-password', code: '123456' });
  expect(screen.queryByLabelText('Password')).toBeNull();
});
it('keeps failed confirmation open and Escape cancels without retry', async () => {
  const send = vi.fn(async () => new Response(JSON.stringify({ detail: 'That code was already used.' }), { status: 403 }));
  const result = requestStepUp({ methods: ['totp'], send });
  const dialog = await screen.findByRole('dialog');
  fireEvent.submit(dialog);
  expect(await screen.findByRole('alert')).toHaveTextContent('That code was already used.');
  await waitFor(() => expect(screen.getByRole('button', { name: 'Cancel' })).not.toBeDisabled());
  fireEvent.keyDown(dialog, { key: 'Escape' });
  await expect(result).resolves.toBe(false);
  expect(send).toHaveBeenCalledTimes(1);
});
it('uses a user-verified passkey', async () => {
  const proof = { credentialId: 'id', clientDataJson: 'data', authenticatorData: 'auth', signature: 'sig', userHandle: null };
  vi.mocked(assertPasskey).mockResolvedValue(proof);
  const send = vi.fn(async (_body: Record<string, unknown>, begin?: boolean) => new Response(JSON.stringify(begin ? { challenge: 'c', rpId: 'mail.example', credentialIds: ['id'] } : {}), { status: 200 }));
  const result = requestStepUp({ methods: ['passkey'], send });
  fireEvent.click(await screen.findByRole('button', { name: 'Use a passkey' }));
  await expect(result).resolves.toBe(true);
  expect(assertPasskey).toHaveBeenCalledWith(expect.objectContaining({ userVerification: 'required' }));
  expect(send).toHaveBeenLastCalledWith({ passkey: proof });
});
