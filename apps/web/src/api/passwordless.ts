import { apiFetch } from './http.ts';
import { basePath } from './basePath.ts';
export async function passwordlessAvailable(): Promise<boolean> {
  const response = await apiFetch(`${basePath()}/api/login/methods`, { credentials: 'same-origin' });
  if (!response.ok) return false;
  const body = await response.json() as { passwordless?: unknown };
  return body.passwordless === true && typeof navigator.credentials?.get === 'function';
}
export async function signInWithPasskey(native = false): Promise<void> {
  const begin = await apiFetch(`${basePath()}/api/login/passkey/begin`, {
    method: 'POST', credentials: 'same-origin', headers: { 'Content-Type': 'application/json' }, body: '{}',
  });
  if (!begin.ok) throw new Error('Passkey sign-in is unavailable.');
  const { assertPasskey } = await import('../screens/Settings/webauthn.ts');
  const challenge = await begin.json() as Parameters<typeof assertPasskey>[0] & { pendingToken: string };
  const assertion = await assertPasskey({ ...challenge, userVerification: 'required' });
  const finish = await apiFetch(`${basePath()}/api/login/passkey/finish`, {
    method: 'POST', credentials: 'same-origin', headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ ...assertion, pendingToken: challenge.pendingToken, ...(native ? { clientType: 'native' } : {}) }),
  });
  if (!finish.ok) throw new Error('Passkey sign-in did not verify.');
}
