import { For, Match, Show, Switch, createMemo, createSignal, onMount, untrack, useContext, type JSX } from 'solid-js';
import { TAG_PROVIDERS, analyticsTagApi, type AnalyticsTagApi, type AnalyticsTagConfig, type AnalyticsTagTest, type AnalyticsTagView, type TagProvider } from '../../api/admin/analyticsTag.ts';
import { formatNumber, loadCatalog, t } from '../../i18n';
import { AsyncPending } from '../../components/AsyncState.tsx';
import { Icon } from '../../components/Icon.tsx';
import { AdminSubsection } from './AdminSubsection.tsx';
import { ShellContext } from './shell/context.ts';
import { useDirtyGuard } from './shell/useDirtyGuard.ts';
import { describeAdminError } from './shell/Feedback.tsx';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';
import * as tag from './analyticsTag.css.ts';

/** Provider names are product names, the same in every language. */
const PROVIDER_NAMES: Record<TagProvider, string> = {
  plausible: 'Plausible', matomo: 'Matomo', umami: 'Umami', fathom: 'Fathom', posthog: 'PostHog',
  'simple-analytics': 'Simple Analytics', cloudflare: 'Cloudflare Web Analytics', custom: '',
};
const HOSTED: ReadonlySet<TagProvider> = new Set(['plausible', 'matomo', 'umami']);
const DEFAULT_HOST: Partial<Record<TagProvider, string>> = { plausible: 'https://plausible.io', umami: 'https://cloud.umami.is' };

function httpsOrigin(raw: string): string | null {
  try { const url = new URL(raw.trim()); return url.protocol === 'https:' ? url.origin : null; } catch { return null; }
}
const lines = (value: string): string[] => value.split(/\r?\n/).map(line => line.trim()).filter(Boolean);
const normalizeOrigin = (value: string): string => value.trim().toLowerCase().replace(/\/$/, '').replace(/:443$/, '');

/**
 * The CSP additions this form would produce, computed as the server does for the
 * presets, and from the declared origins for a custom tag. A preview only: the
 * server validates on save and its result is what the shell receives.
 */
export function cspPreview(config: AnalyticsTagConfig): { scriptSrc: string[]; connectSrc: string[]; imgSrc: string[] } | null {
  const fixed = (script: string[], connect: string[], img: string[] = []) => ({ scriptSrc: script, connectSrc: connect, imgSrc: img });
  const host = (fallback?: string): string | null => httpsOrigin(config.host.trim() || fallback || '');
  switch (config.provider) {
    case 'plausible': { const o = host(DEFAULT_HOST.plausible); return o ? fixed([o], [o]) : null; }
    case 'umami': { const o = host(DEFAULT_HOST.umami); return o ? fixed([o], [o]) : null; }
    case 'matomo': { const o = host(); return o ? fixed([o], [o], [o]) : null; }
    case 'fathom': return fixed(['https://cdn.usefathom.com'], ['https://cdn.usefathom.com'], ['https://cdn.usefathom.com']);
    case 'posthog': { const r = config.region === 'eu' ? 'eu' : 'us'; const api = `https://${r}.i.posthog.com`, assets = `https://${r}-assets.i.posthog.com`; return fixed([assets, api], [api, assets]); }
    case 'simple-analytics': return fixed(['https://scripts.simpleanalyticscdn.com'], ['https://queue.simpleanalyticscdn.com'], ['https://queue.simpleanalyticscdn.com']);
    case 'cloudflare': return fixed(['https://static.cloudflareinsights.com'], ['https://cloudflareinsights.com']);
    case 'custom': {
      const uniq = (list: string[]) => [...new Set(list.map(normalizeOrigin))];
      return fixed(uniq(config.scriptOrigins), uniq(config.connectOrigins), uniq(config.imgOrigins));
    }
  }
}

export function AnalyticsTag(props: { api?: AnalyticsTagApi }): JSX.Element {
  const api = () => props.api ?? analyticsTagApi;
  const shell = useContext(ShellContext);
  const writable = () => shell?.can.can('integrations.write') === true;
  const [view, setView] = createSignal<AnalyticsTagView>();
  const [config, setConfig] = createSignal<AnalyticsTagConfig>();
  const [etag, setEtag] = createSignal('');
  const [baseline, setBaseline] = createSignal('');
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal('');
  const [saved, setSaved] = createSignal(false);
  const [acknowledged, setAcknowledged] = createSignal(false);
  const [testing, setTesting] = createSignal(false);
  const [tested, setTested] = createSignal<AnalyticsTagTest>();
  const [testError, setTestError] = createSignal('');
  const dirty = () => !!config() && JSON.stringify(config()) !== baseline();
  useDirtyGuard(dirty);
  const disabled = () => !writable() || busy();
  const preview = createMemo(() => { const value = config(); return value ? cspPreview(value) : null; });
  const servedPath = (): string | undefined => { const v = view(); return v?.config.enabled && v.preview.valid ? v.preview.path : undefined; };
  const snippetBytes = () => new TextEncoder().encode(config()?.snippet ?? '').length;
  // Every provider's script runs with full page access (t63 F3), presets included.
  const needsAcknowledgement = () => config()?.enabled === true && dirty();

  function accept(result: { data: AnalyticsTagView; etag: string }): void {
    setView(result.data); setConfig(result.data.config); setBaseline(JSON.stringify(result.data.config)); setEtag(result.etag); setAcknowledged(false);
  }
  async function reload(): Promise<void> {
    setBusy(true); setError('');
    try { accept(await api().read()); } catch (e) { setError(describeAdminError(e)); } finally { setBusy(false); }
  }
  async function save(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    const value = config(); if (!value || busy()) return;
    setBusy(true); setError(''); setSaved(false);
    try {
      accept(await api().save({ ...value, ...(needsAcknowledgement() ? { acknowledgeFullAccess: acknowledged() } : {}) }, etag()));
      setSaved(true);
    } catch (e) { setError(describeAdminError(e)); } finally { setBusy(false); }
  }
  async function runTest(): Promise<void> {
    const value = config(); if (!value) return;
    setTesting(true); setTested(); setTestError('');
    try { setTested(await api().test(value)); } catch (e) { setTestError(describeAdminError(e)); } finally { setTesting(false); }
  }
  const change = (patch: Partial<AnalyticsTagConfig>): void => { setSaved(false); setTested(); setConfig(value => value && ({ ...value, ...patch })); };
  const siteLabel = (provider: TagProvider): string => t(`admin-analytics-tag-site-${provider}`);
  onMount(() => { void loadCatalog('admin-analytics'); void reload(); });

  return <AdminSubsection title={t('admin-analytics-tag-title')} description={t('admin-analytics-tag-help')}>
    <ul class={tag.scope} aria-label={t('admin-analytics-tag-scope')}>
      <li>{t('admin-analytics-tag-scope-consent')}</li>
      <li>{t('admin-analytics-tag-scope-consent-own')}</li>
      <li>{t('admin-analytics-tag-scope-never')}</li>
      <li>{t('admin-analytics-tag-scope-privacy')}</li>
    </ul>
    <Show when={error()}><p role="alert" class={css.error}>{error()}</p></Show>
    <Show when={saved()}><p role="status">{t('admin-analytics-tag-saved')}</p></Show>
    <Show when={config()} fallback={<Show when={busy()} fallback={<button class="btn" type="button" onClick={() => void reload()}>{t('admin-analytics-reload')}</button>}><AsyncPending /></Show>}>{value => <form class={css.subsectionBody} onSubmit={event => void save(event)} aria-label={t('admin-analytics-tag-title')}>
      <label class={ux.checkboxRow}><input type="checkbox" checked={value().enabled} disabled={disabled()} onChange={event => change({ enabled: event.currentTarget.checked })} /> {t('admin-analytics-tag-enabled')}</label>
      <label class="field"><span>{t('admin-analytics-tag-provider')}</span>
        <select class={ux.control} value={value().provider} disabled={disabled()} onChange={event => change({ provider: event.currentTarget.value as TagProvider })}>
          <For each={TAG_PROVIDERS}>{provider => <option value={provider}>{provider === 'custom' ? t('admin-analytics-tag-custom') : PROVIDER_NAMES[provider]}</option>}</For>
        </select>
      </label>
      <Show when={value().provider !== 'custom'} fallback={<CustomFields value={value()} disabled={disabled()} change={change} snippetBytes={snippetBytes()} max={view()?.maxSnippetBytes ?? 32768} />}>
        <label class="field"><span>{siteLabel(value().provider)}</span>
          <input class={ux.control} maxlength={128} value={value().siteId} required={value().enabled && value().provider !== 'simple-analytics'} disabled={disabled()} onInput={event => change({ siteId: event.currentTarget.value })} />
          <small>{t(`admin-analytics-tag-site-${value().provider}-help`)}</small>
        </label>
        <Show when={HOSTED.has(value().provider)}>
          <label class="field"><span>{t('admin-analytics-tag-host')}</span>
            <input class={ux.control} type="url" maxlength={512} value={value().host} placeholder={DEFAULT_HOST[value().provider] ?? 'https://matomo.example.com'} required={value().enabled && value().provider === 'matomo'} disabled={disabled()} onInput={event => change({ host: event.currentTarget.value })} />
            <small>{t('admin-analytics-tag-host-help')}</small>
          </label>
        </Show>
        <Show when={value().provider === 'posthog'}>
          <label class="field"><span>{t('admin-analytics-tag-region')}</span>
            <select class={ux.control} value={value().region === 'eu' ? 'eu' : 'us'} disabled={disabled()} onChange={event => change({ region: event.currentTarget.value })}>
              <option value="us">{t('admin-analytics-tag-region-us')}</option><option value="eu">{t('admin-analytics-tag-region-eu')}</option>
            </select>
          </label>
        </Show>
        <p class={tag.warning} role="note"><Icon name="info" />{t('admin-analytics-tag-preset-note')}</p>
        <p class={tag.warning} role="note"><Icon name="alert" /><strong>{t('admin-analytics-tag-preset-warning')}</strong></p>
      </Show>

      <section aria-labelledby="analytics-tag-csp">
        <h4 id="analytics-tag-csp" class={css.subHeading}>{t('admin-analytics-tag-csp')}</h4>
        <p class={css.note}>{t('admin-analytics-tag-csp-help')}</p>
        <Show when={preview()} fallback={<p class={css.note}>{t('admin-analytics-tag-csp-incomplete')}</p>}>{csp => <ul class={tag.directives} data-testid="analytics-tag-csp">
          <For each={[['script-src', csp().scriptSrc], ['connect-src', csp().connectSrc], ['img-src', csp().imgSrc]] as const}>{([name, list]) =>
            <li class={tag.directive}><span class={tag.directiveName}>{name}</span> {list.length ? `+ ${list.join(' ')}` : t('admin-analytics-tag-csp-none')}</li>}</For>
        </ul>}</Show>
        <Show when={servedPath()}>{path => <p class={css.note}>{t('admin-analytics-tag-served', { path: path() })}</p>}</Show>
      </section>

      <Show when={needsAcknowledgement()}>
        <label class={ux.checkboxRow}><input type="checkbox" checked={acknowledged()} disabled={disabled()} onChange={event => setAcknowledged(event.currentTarget.checked)} /> {t('admin-analytics-tag-acknowledge')}</label>
      </Show>
      <div class={tag.actions}>
        <button class="btn btn--primary" type="submit" disabled={disabled() || !dirty() || (needsAcknowledgement() && !acknowledged())}>{t('admin-analytics-tag-save')}</button>
        <button class="btn" type="button" disabled={busy() || testing() || !writable()} data-tooltip={t('admin-analytics-tag-test-tooltip')} onClick={() => void runTest()}><Icon name="diagnostics" /> {t('admin-analytics-tag-test')}</button>
        <button class="btn" type="button" disabled={busy()} onClick={() => void reload()}>{t('admin-analytics-reload')}</button>
      </div>
      <Show when={testing()}><AsyncPending /></Show>
      <Show when={testError()}><p role="alert" class={css.error}>{testError()}</p></Show>
      <Show when={tested()}>{result => <ul class={tag.results} aria-label={t('admin-analytics-tag-test-results')}>
        <li class={tag.result}><Icon name={result().served ? 'check' : 'info'} />{t(result().served ? 'admin-analytics-tag-test-served' : 'admin-analytics-tag-test-unsaved', { path: result().path, bytes: formatNumber(result().bytes) })}</li>
        <For each={result().fetches}>{fetch => <li class={tag.result}><Icon name={fetch.ok ? 'check' : 'alert'} />
          <Switch><Match when={fetch.ok}>{t('admin-analytics-tag-test-ok', { url: fetch.url, bytes: formatNumber(fetch.bytes ?? 0), type: fetch.contentType ?? '?' })}</Match>
            <Match when={!fetch.ok}>{t('admin-analytics-tag-test-failed', { url: fetch.url, reason: fetch.error ?? '?' })}</Match></Switch></li>}</For>
      </ul>}</Show>
    </form>}</Show>
  </AdminSubsection>;
}

function CustomFields(props: { value: AnalyticsTagConfig; disabled: boolean; change: (patch: Partial<AnalyticsTagConfig>) => void; snippetBytes: number; max: number }): JSX.Element {
  // Uncontrolled: the text stays as typed (blank lines included) and only its
  // non-empty lines reach the configuration.
  const originField = (key: 'scriptOrigins' | 'connectOrigins' | 'imgOrigins', label: string, help: string) => {
    const initial = props.value[key].join('\n');
    return <label class="field"><span>{t(label)}</span>
      <textarea ref={el => { el.value = initial; }} class={`${ux.control} ${tag.origins}`} rows={3} spellcheck={false} disabled={props.disabled} placeholder="https://stats.example.com" onInput={event => props.change({ [key]: lines(event.currentTarget.value) })} />
      <small>{t(help)}</small>
    </label>;
  };
  // Built once, outside a tracked JSX insert, so typing never rebuilds them.
  const originFields = untrack(() => [
    originField('scriptOrigins', 'admin-analytics-tag-script-origins', 'admin-analytics-tag-script-origins-help'),
    originField('connectOrigins', 'admin-analytics-tag-connect-origins', 'admin-analytics-tag-connect-origins-help'),
    originField('imgOrigins', 'admin-analytics-tag-img-origins', 'admin-analytics-tag-img-origins-help'),
  ]);
  return <>
    <p class={tag.warning} role="note"><Icon name="alert" /><strong>{t('admin-analytics-tag-custom-warning')}</strong></p>
    <fieldset class={ux.fieldset} disabled={props.disabled}>
      <legend class={ux.legend}>{t('admin-analytics-tag-kind')}</legend>
      <label class={ux.checkboxRow}><input type="radio" name="analytics-tag-kind" checked={props.value.customKind === 'url'} onChange={() => props.change({ customKind: 'url' })} /> {t('admin-analytics-tag-kind-url')}</label>
      <label class={ux.checkboxRow}><input type="radio" name="analytics-tag-kind" checked={props.value.customKind === 'inline'} onChange={() => props.change({ customKind: 'inline' })} /> {t('admin-analytics-tag-kind-inline')}</label>
    </fieldset>
    <Show when={props.value.customKind === 'url'} fallback={
      <label class="field"><span>{t('admin-analytics-tag-snippet')}</span>
        <textarea class={`${ux.control} ${tag.snippet}`} rows={8} spellcheck={false} value={props.value.snippet} disabled={props.disabled} onInput={event => props.change({ snippet: event.currentTarget.value })} />
        <small aria-live="polite">{t('admin-analytics-tag-snippet-help', { used: formatNumber(props.snippetBytes), max: formatNumber(props.max) })}</small>
      </label>}>
      <label class="field"><span>{t('admin-analytics-tag-script-url')}</span>
        <input class={ux.control} type="url" maxlength={2048} value={props.value.scriptUrl} placeholder="https://cdn.example.com/tag.js" disabled={props.disabled} onInput={event => props.change({ scriptUrl: event.currentTarget.value })} />
        <small>{t('admin-analytics-tag-script-url-help')}</small>
      </label>
    </Show>
    {originFields}
  </>;
}
