import { createSignal, onMount, Show, type JSX } from 'solid-js';
import { ApiError } from '../api/client.ts';
import { useApp } from '../state/context.ts';
import { t, loadCatalog } from '../i18n';

export function Login(): JSX.Element {
  const app = useApp();
  onMount(() => void loadCatalog('auth'));
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
        setError(t('auth-invalid-credentials'));
      } else {
        setError(t('auth-unreachable'));
      }
    } finally {
      setBusy(false);
    }
  }

  return (
    <main class="login">
      <form class="login__card" onSubmit={(e) => void onSubmit(e)} aria-label={t('auth-sign-in')}>
        <h1 class="login__title">{t('auth-app-name')}</h1>
        <label class="field">
          <span>{t('auth-jmap-url')}</span>
          <input
            type="url"
            required
            placeholder={t('auth-jmap-url-placeholder')}
            value={jmapUrl()}
            onInput={(e) => setJmapUrl(e.currentTarget.value)}
          />
        </label>
        <label class="field">
          <span>{t('auth-username')}</span>
          <input
            type="text"
            required
            autocomplete="username"
            value={username()}
            onInput={(e) => setUsername(e.currentTarget.value)}
          />
        </label>
        <label class="field">
          <span>{t('auth-password')}</span>
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
          {busy() ? t('auth-signing-in') : t('auth-sign-in')}
        </button>
        <p class="login__hint">{t('auth-mock-hint')}</p>
      </form>
    </main>
  );
}
