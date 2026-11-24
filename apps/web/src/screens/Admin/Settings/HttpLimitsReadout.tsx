// HTTP limits readout, shown above the Maintenance settings: what each
// server.http.* value resolved to on this server process (Auto, the console or
// the environment) and the readings Auto used. Read from the HTTP admission row
// of the deployment security assessment; nothing is shown that the server does
// not report.

import { createResource, For, Show, type JSX } from 'solid-js';
import { formatNumber, t } from '../../../i18n';
import { adminRequest } from '../shell/adminApi.ts';
import { keySlug } from './keys.ts';

interface Entry {
  value?: number | boolean;
  source?: string;
  capped?: boolean;
}
export interface HttpLimitsEffective {
  connections?: Entry;
  requests?: Entry;
  http2Streams?: Entry;
  queueWaitMs?: Entry;
  queueMax?: Entry;
  adaptive?: Entry;
  adaptiveFloor?: Entry;
}
interface Assessment {
  checks?: { id?: string; value?: { admission?: { effective?: HttpLimitsEffective; inputs?: string } | null } }[];
}

const ROWS: [keyof HttpLimitsEffective, string][] = [
  ['connections', 'server.http.maxConnections'],
  ['requests', 'server.http.maxRequests'],
  ['http2Streams', 'server.http.http2MaxStreams'],
  ['queueWaitMs', 'server.http.requestQueueMs'],
  ['queueMax', 'server.http.requestQueueMax'],
  ['adaptive', 'server.http.adaptiveConcurrency'],
  ['adaptiveFloor', 'server.http.adaptiveFloor'],
];

const shown = (v: unknown): string =>
  typeof v === 'number' ? formatNumber(v) : typeof v === 'boolean' ? t(v ? 'admin-http-limits-on' : 'admin-http-limits-off') : t('admin-image-proxy-unknown');
const source = (s: string | undefined): string => {
  switch (s) {
    case 'auto':
      return t('admin-http-limits-source-auto');
    case 'console':
      return t('admin-http-limits-source-console');
    case 'explicit':
      return t('admin-http-limits-source-environment');
    default:
      return t('admin-http-limits-source-default');
  }
};

export function HttpLimitsReadout(props: { load?: () => Promise<Assessment> }): JSX.Element {
  const [data] = createResource<Assessment | undefined>(async () => (await (props.load ?? (async () => (await adminRequest<Assessment>('/security/deployment')).data))()));
  const admission = () => (data.error ? undefined : data.latest?.checks?.find((c) => c.id === 'httpAdmission')?.value?.admission ?? undefined);
  return (
    <section data-testid="http-limits-readout" aria-label={t('admin-http-limits-title')}>
      <h3>{t('admin-http-limits-title')}</h3>
      <Show when={admission()?.effective} fallback={<p>{t('admin-http-limits-unavailable')}</p>}>
        {(effective) => (
          <>
            <ul>
              <For each={ROWS}>
                {([field, key]) => (
                  <li data-testid={`http-limits-${field}`}>
                    {t('admin-http-limits-row', {
                      name: t(`admin-settings-key-${keySlug(key)}`),
                      value: shown(effective()[field]?.value),
                      source: source(effective()[field]?.source),
                    })}
                    <Show when={effective()[field]?.capped}> {t('admin-http-limits-capped')}</Show>
                  </li>
                )}
              </For>
            </ul>
            <Show when={admission()?.inputs}>{(inputs) => <p data-testid="http-limits-inputs">{t('admin-http-limits-inputs', { inputs: inputs() })}</p>}</Show>
            <p>{t('admin-http-limits-per-process')}</p>
          </>
        )}
      </Show>
    </section>
  );
}
