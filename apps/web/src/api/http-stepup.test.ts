import { afterEach, beforeEach, expect, it, vi } from 'vitest';
import { apiFetch, setNativeTransport } from './http.ts';
import { requestStepUp } from '../components/StepUpDialog.tsx';
vi.mock('../components/StepUpDialog.tsx', () => ({ requestStepUp: vi.fn() }));
const prompt = vi.mocked(requestStepUp);
const refused = (): Response => new Response(JSON.stringify({ code: 'step_up_required', methods: ['password'] }), { status: 403 });
const success = (): Response => new Response('{}', { status: 200 });
const fetchMock = vi.fn<typeof fetch>();
beforeEach(() => { vi.stubGlobal('fetch', fetchMock); fetchMock.mockReset(); prompt.mockReset(); setNativeTransport(null); });
afterEach(() => { vi.unstubAllGlobals(); setNativeTransport(null); });
it('confirms once and retries the original body once', async () => {
  fetchMock.mockResolvedValueOnce(refused()).mockResolvedValueOnce(success()).mockResolvedValueOnce(success());
  prompt.mockImplementation(async ({ send }) => (await send({ password: 'secret' })).ok);
  expect((await apiFetch('/api/keys', { method: 'POST', body: 'original' })).status).toBe(200);
  expect(fetchMock.mock.calls.map(([path]) => path)).toEqual(['/api/keys', '/api/security/step-up', '/api/keys']);
  expect(fetchMock.mock.calls[2]?.[1]?.body).toBe('original');
  expect(prompt).toHaveBeenCalledTimes(1);
});
it('does not retry a cancelled request or recursively prompt on another refusal', async () => {
  fetchMock.mockImplementation(async () => refused());
  prompt.mockResolvedValueOnce(false);
  expect((await apiFetch('/api/keys', { method: 'POST' })).status).toBe(403);
  expect(fetchMock).toHaveBeenCalledTimes(1);
  prompt.mockResolvedValueOnce(true);
  await apiFetch('/api/keys', { method: 'POST' });
  expect(fetchMock).toHaveBeenCalledTimes(3);
  expect(prompt).toHaveBeenCalledTimes(2);
});
it('never requests browser proof for bearer, omitted cookies or another origin', async () => {
  fetchMock.mockImplementation(async () => refused());
  await apiFetch('/api/keys', { headers: { authorization: 'Bearer key' } });
  await apiFetch('/api/keys', { credentials: 'omit' });
  await apiFetch('https://untrusted.example/api/keys');
  expect(prompt).not.toHaveBeenCalled();
});
it('shares one pending proof between parallel refused actions', async () => {
  fetchMock.mockImplementation(async () => refused());
  let done: (value: boolean) => void = () => undefined;
  prompt.mockImplementation(() => new Promise((resolve) => { done = resolve; }));
  const first = apiFetch('/api/keys', { method: 'POST' });
  const second = apiFetch('/api/keys', { method: 'POST' });
  await vi.waitFor(() => expect(prompt).toHaveBeenCalledTimes(1));
  done(true);
  await Promise.all([first, second]);
  expect(fetchMock).toHaveBeenCalledTimes(4);
});
it('does not retry after the caller aborts while confirmation is open', async () => {
  fetchMock.mockImplementation(async () => refused());
  const controller = new AbortController();
  prompt.mockImplementation(async () => { controller.abort(); return true; });
  await apiFetch('/api/keys', { signal: controller.signal });
  expect(fetchMock).toHaveBeenCalledTimes(1);
});
it('confirms a native session through its configured server and keeps bearer auth', async () => {
  setNativeTransport({ base: () => 'https://mail.example', token: async () => 'session', setToken: async () => undefined });
  fetchMock.mockResolvedValueOnce(refused()).mockResolvedValueOnce(success()).mockResolvedValueOnce(success());
  prompt.mockImplementation(async ({ send }) => (await send({ password: 'secret' })).ok);
  expect((await apiFetch('/api/keys', { method: 'POST', body: 'original' })).status).toBe(200);
  expect(fetchMock.mock.calls.map(([path]) => path)).toEqual(['https://mail.example/api/keys', 'https://mail.example/api/security/step-up', 'https://mail.example/api/keys']);
  expect(fetchMock.mock.calls.every(([, init]) => new Headers(init?.headers).get('authorization') === 'Bearer session')).toBe(true);
});
it('never retries a native action after the signed-in account changes', async () => {
  let token = 'original-session';
  setNativeTransport({ base: () => 'https://mail.example', token: async () => token, setToken: async () => undefined });
  fetchMock.mockImplementation(async () => refused());
  prompt.mockImplementation(async () => { token = 'another-account'; return true; });
  expect((await apiFetch('/api/keys', { method: 'POST' })).status).toBe(403);
  expect(fetchMock).toHaveBeenCalledTimes(1);
});
