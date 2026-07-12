import { createSignal, Show, type JSX } from 'solid-js';
import { ApiError } from '../api/client.ts';
import { useApp } from '../state/context.ts';

export function Login(): JSX.Element {
  const app = useApp();
  const [jmapUrl, setJmapUrl] = createSignal('');
  const [username, setUsername] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [error, setError] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);

  async function onSubmit(e: Event): Promise<void> {
    e.preventDefault();
    setError(null);
    setBusy(true);
    try {
      await app.login({ jmapUrl: jmapUrl(), username: username(), password: password() });
    } catch (err) {
      if (err instanceof ApiError && err.status === 401) {
        setError('Invalid credentials');
      } else {
        setError('Could not reach the server');
      }
    } finally {
      setBusy(false);
    }
  }

  return (
    <main class="login">
      <form class="login__card" onSubmit={(e) => void onSubmit(e)}>
        <h1 class="login__title">Mailwoman</h1>
        <label class="field">
          <span>JMAP server URL</span>
          <input
            type="url"
            required
            placeholder="https://jmap.example.org"
            value={jmapUrl()}
            onInput={(e) => setJmapUrl(e.currentTarget.value)}
          />
        </label>
        <label class="field">
          <span>Username</span>
          <input
            type="text"
            required
            autocomplete="username"
            value={username()}
            onInput={(e) => setUsername(e.currentTarget.value)}
          />
        </label>
        <label class="field">
          <span>Password</span>
          <input
            type="password"
            required
            autocomplete="current-password"
            value={password()}
            onInput={(e) => setPassword(e.currentTarget.value)}
          />
        </label>
        <Show when={error()}>
          <p class="login__error" role="alert">
            {error()}
          </p>
        </Show>
        <button type="submit" class="btn btn--primary" disabled={busy()}>
          {busy() ? 'Signing in…' : 'Sign in'}
        </button>
        <p class="login__hint">Mock account: testuser@example.org / testpass</p>
      </form>
    </main>
  );
}
