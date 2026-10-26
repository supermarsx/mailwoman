import { createEffect, createSignal, onCleanup, onMount, Show, type JSX } from 'solid-js';
import type { AltchaWidgetElement } from 'altcha';
import Pbkdf2Worker from 'altcha/workers/pbkdf2?worker';
import 'altcha/altcha.css';
import { apiFetch } from '../api/http.ts';
import { transportBase } from '../api/transport.ts';
import { t, loadCatalog } from '../i18n';
import {verification} from './signinChallenge.css.ts';

let load: Promise<void> | undefined;
function loadWidget(): Promise<void> {
  return load ??= import('altcha/external').then(() => {
    globalThis.$altcha.algorithms.set('PBKDF2/SHA-256', () => new Pbkdf2Worker());
  }).catch(error => { load = undefined; throw error; });
}
export interface SigninChallengeProps {
  surface: 'mailbox' | 'admin';
  reset: number;
  onProof: (proof: string | undefined) => void;
}
/** Local worker and same-server challenge only. No Sentinel, cookies or interaction telemetry. */
export function SigninChallenge(props: SigninChallengeProps): JSX.Element {
  let host!: HTMLDivElement;
  let widget: AltchaWidgetElement | undefined;
  let disposed = false;
  const [loading, setLoading] = createSignal(true);
  const [failed, setFailed] = createSignal(false);
  createEffect(() => { void props.reset; props.onProof(undefined); widget?.reset(); });
  onMount(() => {
    const challenge = `${transportBase()}/api/signin/challenge?surface=${props.surface}`;
    void Promise.all([loadCatalog('auth'), loadWidget()]).then(async () => {
      if (disposed) return;
      const element = document.createElement('altcha-widget');
      element.setAttribute('configuration', JSON.stringify({ humanInteractionSignature: false, auto: 'off', workers: 1, type: 'native' }));
      widget = element;
      element.addEventListener('verified', event => {
        const payload = (event as CustomEvent<{ payload?: unknown }>).detail?.payload;
        props.onProof(typeof payload === 'string' && payload.length <= 8192 ? payload : undefined);
      });
      element.addEventListener('expired', () => props.onProof(undefined));
      element.addEventListener('statechange', event => {
        const state = (event as CustomEvent<{ state?: string }>).detail?.state;
        if (state !== 'verified') props.onProof(undefined);
        setFailed(state === 'error');
      });
      element.addEventListener('load', () => {
        void Promise.resolve(element.configure({
          challenge, auto: 'off', workers: 1, type: 'native', humanInteractionSignature: false,
          fetch: async (input, init) => {
            const url = typeof input === 'string' ? input : input instanceof URL ? input.href : input.url;
            if (new URL(url, location.href).href !== new URL(challenge, location.href).href) throw new Error('Unexpected verification endpoint');
            return apiFetch(challenge, { ...init, cache: 'no-store', redirect: 'error' });
          },
        })).then(() => { if (!disposed) setLoading(false); }).catch(() => { if (!disposed) { setLoading(false); setFailed(true); } });
      }, { once: true });
      host.append(element);
    }).catch(() => { if (!disposed) { setLoading(false); setFailed(true); } });
  });
  onCleanup(() => { disposed = true; widget?.remove(); props.onProof(undefined); });
  return <section class={verification} aria-label={t('auth-verification-title')}>
    <Show when={loading()}><p role="status">{t('auth-verification-loading')}</p></Show>
    <div ref={host} />
    <Show when={failed()}><p role="alert">{t('auth-verification-error')}</p></Show>
  </section>;
}
