import { storageAuthorityHeaders } from './storageAuthority.ts';
import { apiFetch } from './http.ts';
import { withBase } from './basePath.ts';

const SCREENS = ['mail', 'calendar', 'tasks', 'contacts', 'notes', 'settings'] as const;
export type AnalyticsScreen = typeof SCREENS[number];
export function analyticsScreen(value: string): AnalyticsScreen | null {
  return (SCREENS as readonly string[]).includes(value) ? value as AnalyticsScreen : null;
}
export function analyticsPrivacySignal(nav: Pick<Navigator, 'doNotTrack'> & {globalPrivacyControl?: boolean} = navigator): boolean {
  return nav.doNotTrack === '1' || nav.globalPrivacyControl === true;
}
export interface AnalyticsTransport {
  enabled(signal: AbortSignal): Promise<boolean>;
  send(screen: AnalyticsScreen, signal: AbortSignal): Promise<void>;
}
const transport: AnalyticsTransport = {
  async enabled(signal) {
    const response = await apiFetch(withBase('/api/analytics/status'), {signal, headers:storageAuthorityHeaders()});
    if (!response.ok) return false;
    const body = await response.json() as {enabled?: boolean; consent?: boolean; privacySignal?: boolean};
    return body.enabled === true && body.consent === true && body.privacySignal === false;
  },
  async send(screen, signal) {
    await apiFetch(withBase('/api/analytics/events'), {method:'POST', signal, headers:{...storageAuthorityHeaders(),'content-type':'application/json'}, body:JSON.stringify({screen})});
  },
};

/** Ephemeral and best effort: no persistent queue, retries, URLs or user IDs.
 * Account generations only cancel local work and never enter a request body. */
export function createAnalyticsReporter(api: AnalyticsTransport = transport) {
  let controller: AbortController | null = null;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let previous = '';
  function stop(): void {
    clearTimeout(timer); timer = undefined;
    controller?.abort(); controller = null;
    previous = '';
  }
  return {
    stop,
    update(generation: number | null, screen: AnalyticsScreen | null, consent: boolean, privacySignal: boolean): void {
      if (generation === null || !screen || !consent || privacySignal) { stop(); return; }
      const key = `${generation}:${screen}`;
      if (key === previous) return;
      stop(); previous = key;
      const current = new AbortController(); controller = current;
      timer = setTimeout(() => {
        timer = undefined;
        // The whole status + send attempt is bounded and has no retry.
        const deadline = setTimeout(() => current.abort(), 5000);
        void api.enabled(current.signal).then(enabled => {
          if (enabled && controller === current && !current.signal.aborted) return api.send(screen, current.signal);
        }).catch(() => { /* Counts are optional; never disrupt application use. */ }).finally(() => clearTimeout(deadline));
      }, 750);
    },
  };
}
