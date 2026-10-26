import { createEffect, createResource, createSignal, For, on, onMount, Show, type JSX } from 'solid-js';
import { AppContext } from '../../state/context.ts';
import { createAppState } from '../../state/store.ts';
import { createConfiguredClient } from '../../api/transport.ts';
import { apiFetch } from '../../api/http.ts';
import { basePath } from '../../api/basePath.ts';
import { Login } from '../Login.tsx';
import { loadCatalog, t } from '../../i18n';
import * as css from '../Consent/consent.css.ts';

type Context = { loginRequired?: boolean; reauthenticationRequired?: boolean; clientName?: string; clientId?: string; account?: string; redirectUri?: string; scopes?: string[]; csrfToken?: string };
export default function IdentityConsent(props: { onRedirect?: (uri: string) => void } = {}): JSX.Element {
  const app = createAppState(createConfiguredClient());
  const query = new URLSearchParams(location.search);
  const logout = query.has('logout');
  const flow = query.get(logout ? 'logout' : 'flow') ?? '';
  const [generation, reload] = createSignal(0);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  onMount(() => { void loadCatalog('auth'); });
  createEffect(on(app.me, () => { reload(value => value + 1); }, { defer: true }));
  const [context, { refetch }] = createResource(generation, async (): Promise<Context> => {
    const response = await apiFetch(`${basePath()}/oidc/${logout ? 'logout/context' : 'context'}?flow=${encodeURIComponent(flow)}`, { credentials: 'same-origin', cache: 'no-store' });
    if (!response.ok) throw new Error(t('auth-identity-expired'));
    return await response.json() as Context;
  });
  async function decide(approve: boolean): Promise<void> {
    if (busy()) return;
    setBusy(true); setError('');
    try {
      const csrf = context()?.csrfToken;
      if (!csrf) throw new Error(t('auth-identity-expired'));
      const response = await apiFetch(`${basePath()}/oidc/${logout ? 'logout' : 'decision'}`, { method: 'POST', credentials: 'same-origin', headers: { 'content-type': 'application/json', 'x-csrf-token': csrf }, body: JSON.stringify({ flow, approve }) });
      if (!response.ok) throw new Error(t('auth-identity-expired'));
      const result = await response.json() as { redirectUri: string };
      // The server returns only the exact registered RP URI or local home path.
      if (props.onRedirect) props.onRedirect(result.redirectUri); else location.assign(result.redirectUri);
    } catch (cause) { setError(cause instanceof Error ? cause.message : t('auth-identity-expired')); setBusy(false); }
  }
  return <AppContext.Provider value={app}>
    <Show when={!context()?.loginRequired} fallback={<><p class={css.prose}>{t('auth-identity-sign-in')}</p><Login returnPath={`/oidc/consent?flow=${encodeURIComponent(flow)}`} /></>}>
      <main class={css.backdrop}><section class={css.card} aria-label={t(logout ? 'auth-identity-sign-out' : 'auth-identity-title')}>
        <h1 class={css.heading}>{t(logout ? 'auth-identity-sign-out' : 'auth-identity-title')}</h1>
        <Show when={context.loading}><p role="status">{t('auth-consent-loading')}</p></Show>
        <Show when={context.error}><p role="alert" class={css.error}>{t('auth-identity-expired')}</p><button type="button" onClick={() => { void refetch(); }}>{t('auth-identity-retry')}</button></Show>
        <Show when={context()}>{value => <>
          <Show when={!logout}><p class={css.client} dir="auto">{value().clientName}</p></Show>
          <p class={css.prose} dir="auto">{value().account}</p>
          <Show when={!logout}><p class={css.prose}>{t('auth-identity-disclosure')}</p><ul class={css.scopeList}><For each={value().scopes}>{scope => <li>{t(`auth-identity-scope-${scope}`)}</li>}</For></ul></Show>
          <Show when={value().redirectUri}><p class={css.meta}>{value().redirectUri}</p></Show>
          <Show when={error()}><p class={css.error} role="alert">{error()}</p></Show>
          <div class={css.actions}><button class={css.deny} disabled={busy()} onClick={() => { void decide(false); }}>{t('auth-identity-deny')}</button><button class={css.grant} disabled={busy()} onClick={() => { void decide(true); }}>{t(logout ? 'auth-identity-sign-out' : 'auth-identity-approve')}</button></div>
        </>}</Show>
      </section></main>
    </Show>
  </AppContext.Provider>;
}
