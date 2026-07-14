// OAuth 2.1 consent screen (SPEC §20.1, plan §3 e8). Shows the requesting client, its
// admin-approval status, and the EXACT scope it is asking for, then lets the resource
// owner grant or deny. On grant the server mints the authorization code and returns the
// redirect target; on deny the server returns the `access_denied` redirect. The client
// never learns the owner's identity from this screen — the session supplies it.
//
// EXPORTED (default) so e11 can register it as a lazy route (e.g. `/oauth/authorize`);
// this file does NOT touch the app router.

import { createSignal, createResource, For, Show, type JSX } from 'solid-js';
import { ConsentService, parseAuthorizeParams, type AuthorizeParams, type ConsentContext, type Fetcher } from './service.ts';
import { scopeFromWire, summarizeScope, UNATTENDED_SEND_DISCLOSURE } from '../../modules/apikeys/index.ts';
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
      setError(e instanceof Error ? e.message : 'could not record your decision');
      setBusy(false);
    }
  }

  return (
    <div class={css.backdrop}>
      <section class={css.card} role="dialog" aria-modal="true" aria-label="Authorize application">
        <h1 class={css.heading}>Authorize access</h1>

        <Show when={ctx()} fallback={<p class={css.prose}>Loading request…</p>}>
          {(c) => {
            const scope = scopeFromWire(c().requestedScope);
            const sendsUnattended = scope.unattendedSend;
            return (
              <>
                <p class={css.prose}>
                  <span class={css.client}>{c().clientName}</span> wants to access your account.
                </p>
                <Show
                  when={c().approved}
                  fallback={
                    <span class={css.unapproved} data-testid="client-unapproved">
                      Unrecognised client — not admin-approved
                    </span>
                  }
                >
                  <span class={css.approved} data-testid="client-approved">
                    Admin-approved client
                  </span>
                </Show>

                <div>
                  <span class={css.subHeading}>It is requesting</span>
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
                  <span class={css.subHeading}>Redirects to</span>
                  <p class={css.meta}>{c().redirectUri}</p>
                  <span class={css.subHeading}>For resource</span>
                  <p class={css.meta}>{c().resource}</p>
                </div>

                <Show when={error() !== ''}>
                  <p class={css.error} role="alert">
                    {error()}
                  </p>
                </Show>

                <div class={css.actions}>
                  <button type="button" class={css.deny} disabled={busy()} onClick={() => void decide(false)}>
                    Deny
                  </button>
                  <button type="button" class={css.grant} disabled={busy()} onClick={() => void decide(true)}>
                    Allow
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
