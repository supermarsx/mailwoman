// OAuth 2.1 consent screen (SPEC §20.1, plan §3 e8). Shows the requesting client, its
// admin-approval status, and the EXACT scope it is asking for, then lets the resource
// owner grant or deny. On grant the server mints the authorization code and returns the
// redirect target; on deny the server returns the `access_denied` redirect. The client
// never learns the owner's identity from this screen — the session supplies it.
//
// EXPORTED (default) so e11 can register it as a lazy route (e.g. `/oauth/authorize`);
// this file does NOT touch the app router.

import { createSignal, createResource, For, onMount, Show, type JSX } from 'solid-js';
import { ConsentService, parseAuthorizeParams, type AuthorizeParams, type ConsentContext, type Fetcher } from './service.ts';
import { scopeFromWire, summarizeScope, UNATTENDED_SEND_DISCLOSURE } from '../../modules/apikeys/index.ts';
import { t, loadCatalog } from '../../i18n';
import { createFocusTrap } from '../../components/a11y';
import * as css from './consent.css.ts';

export interface ConsentScreenProps {
  /** The authorize params. Defaults to parsing `window.location.search`. */
  params?: AuthorizeParams;
  fetcher?: Fetcher;
  /** Tests inject a preloaded context; production fetches. */
  initialContext?: ConsentContext;
  /** Where to send the browser after a decision (default: `location.assign`). */
  onRedirect?: (url: string) => void;
}

export function ConsentScreen(props: ConsentScreenProps): JSX.Element {
  const service = new ConsentService(props.fetcher);
  let card!: HTMLElement;
  onMount(() => void loadCatalog('auth'));
  // The authorization card is a modal dialog covering the viewport: trap focus
  // inside it so keyboard users stay within the grant/deny controls.
  createFocusTrap(() => card);
  const params = (): AuthorizeParams =>
    props.params ?? parseAuthorizeParams(typeof window !== 'undefined' ? window.location.search : '');

  const [ctx] = createResource<ConsentContext>(() => props.initialContext ?? service.context(params()));
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');

  function redirect(url: string): void {
    if (props.onRedirect) props.onRedirect(url);
    else if (typeof window !== 'undefined') window.location.assign(url);
  }

  async function decide(approve: boolean): Promise<void> {
    setError('');
    setBusy(true);
    try {
      const result = await service.decide(params(), approve);
      redirect(result.redirectUri);
    } catch (e) {
      setError(e instanceof Error ? e.message : t('auth-consent-error'));
      setBusy(false);
    }
  }

  return (
    <div class={css.backdrop}>
      <section ref={card} class={css.card} role="dialog" aria-modal="true" aria-label={t('auth-consent-dialog')} tabindex="-1">
        <h1 class={css.heading}>{t('auth-consent-title')}</h1>

        <Show when={ctx()} fallback={<p class={css.prose}>{t('auth-consent-loading')}</p>}>
          {(c) => {
            const scope = scopeFromWire(c().requestedScope);
            const sendsUnattended = scope.unattendedSend;
            return (
              <>
                <p class={css.prose}>
                  <span class={css.client} dir="auto">
                    {c().clientName}
                  </span>{' '}
                  {t('auth-consent-intro')}
                </p>
                <Show
                  when={c().approved}
                  fallback={
                    <span class={css.unapproved} data-testid="client-unapproved">
                      {t('auth-consent-unapproved')}
                    </span>
                  }
                >
                  <span class={css.approved} data-testid="client-approved">
                    {t('auth-consent-approved')}
                  </span>
                </Show>

                <div>
                  <span class={css.subHeading}>{t('auth-consent-requesting')}</span>
                  <ul class={css.scopeList} data-testid="requested-scope">
                    <For each={summarizeScope(scope)}>{(line) => <li>{line}</li>}</For>
                  </ul>
                </div>

                <Show when={sendsUnattended}>
                  <p class={css.prose} data-testid="consent-unattended-send">
                    {UNATTENDED_SEND_DISCLOSURE}
                  </p>
                </Show>

                <div>
                  <span class={css.subHeading}>{t('auth-consent-redirects-to')}</span>
                  <p class={css.meta} dir="auto">
                    {c().redirectUri}
                  </p>
                  <span class={css.subHeading}>{t('auth-consent-for-resource')}</span>
                  <p class={css.meta} dir="auto">
                    {c().resource}
                  </p>
                </div>

                <Show when={error() !== ''}>
                  <p class={css.error} role="alert">
                    {error()}
                  </p>
                </Show>

                <div class={css.actions}>
                  <button type="button" class={css.deny} disabled={busy()} onClick={() => void decide(false)}>
                    {t('auth-consent-deny')}
                  </button>
                  <button type="button" class={css.grant} disabled={busy()} onClick={() => void decide(true)}>
                    {t('auth-consent-allow')}
                  </button>
                </div>
              </>
            );
          }}
        </Show>
      </section>
    </div>
  );
}

export default ConsentScreen;
