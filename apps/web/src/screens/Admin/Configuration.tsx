import { For, Show, createMemo, createSignal, onCleanup, onMount, type JSX } from 'solid-js';
import { loadCatalog, t } from '../../i18n';
import { loadEffectiveConfig, type AdminReadResult, type EffectiveEntry } from './shell/adminApi.ts';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import { AdminSubsection } from './AdminSubsection.tsx';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';
import * as layout from './configuration.css.ts';
import { categoryLabel, settingLabel } from './Settings/labels.ts';
import { Icon } from '../../components/Icon.tsx';
import { AsyncPending } from '../../components/AsyncState.tsx';
import { hrefFor, sectionPath } from './shell/router.ts';

export interface ConfigurationProps { load?: () => Promise<AdminReadResult<EffectiveEntry[]>> }

function displayValue(value: unknown): string {
  if (value === null || value === undefined) return t('admin-config-empty');
  if (typeof value === 'object' && 'set' in value) return t((value as { set: boolean }).set ? 'admin-config-secret-set' : 'admin-config-secret-unset');
  if (typeof value === 'boolean') return t(value ? 'admin-settings-value-on' : 'admin-settings-value-off');
  return typeof value === 'string' ? value : JSON.stringify(value, null, 2);
}

/** A redacted view of the server's report; unavailable runtime values stay unknown. */
export function Configuration(props: ConfigurationProps): JSX.Element {
  const [rows, setRows] = createSignal<EffectiveEntry[]>([]);
  const [query, setQuery] = createSignal(new URLSearchParams(location.hash.split('?')[1] ?? '').get('field') ?? '');
  const [group, setGroup] = createSignal('');
  const [source, setSource] = createSignal('');
  const [limit, setLimit] = createSignal(50);
  const [busy, setBusy] = createSignal(true);
  const [unavailable, setUnavailable] = createSignal(false);
  const [error, setError] = createSignal<string>();
  let current = 0;
  onCleanup(() => { current += 1; });
  async function reload(): Promise<void> {
    const request = ++current;
    setBusy(true);
    setError(undefined);
    try {
      const result = await (props.load ?? loadEffectiveConfig)();
      if (request !== current) return;
      setUnavailable(result.kind === 'unavailable');
      setRows(result.kind === 'ok' ? result.data : []);
    } catch (e) {
      if (request !== current) return;
      setRows([]);
      setError(describeAdminError(e));
    } finally { if (request === current) setBusy(false); }
  }
  onMount(() => { void loadCatalog('admin-settings'); void reload(); });
  const groups = createMemo(() => [...new Set(rows().map((row) => row.group))].sort());
  const filtered = createMemo(() => {
    const needle = query().trim().toLowerCase();
    return rows().filter((row) => (!group() || row.group === group()) && (!source() || row.source === source()) && `${settingLabel(row.id)} ${row.id} ${row.what ?? ''} ${row.searchText ?? ''} ${row.envVar ?? ''}`.toLowerCase().includes(needle));
  });
  return <section class={css.section} aria-label={t('admin-config-title')}>
    <div class={ux.pageHead}>
      <div class={layout.title}><h2 class={css.heading}>{t('admin-config-title')}</h2><p class={css.note}>{t('admin-config-description')}</p></div>
      <button type="button" class={`btn ${ux.iconAction}`} aria-label={t('admin-config-refresh')} title={t('admin-config-refresh')} disabled={busy()} onClick={() => void reload()}><Icon name="refresh" /></button>
    </div>
    <Feedback kind="error" message={error()} />
    <Show when={unavailable()}><p role="status">{t('admin-config-unavailable')}</p></Show>
    <AdminSubsection title={t('admin-config-filters')}><div class={css.grid}>
      <label class="field"><span>{t('admin-config-search')}</span><input type="search" value={query()} onInput={(e) => { setQuery(e.currentTarget.value); setLimit(50); }} /></label>
      <label class="field"><span>{t('admin-config-group')}</span><select value={group()} onChange={(e) => { setGroup(e.currentTarget.value); setLimit(50); }}><option value="">{t('admin-config-all')}</option><For each={groups()}>{(g) => <option value={g}>{categoryLabel(g)}</option>}</For></select></label>
      <label class="field"><span>{t('admin-config-source')}</span><select value={source()} onChange={e => { setSource(e.currentTarget.value); setLimit(50); }}><option value="">{t('admin-config-all-sources')}</option><For each={[...new Set(rows().map(row => row.source))].sort()}>{value => <option value={value}>{t(`admin-config-source-${value}`)}</option>}</For></select></label>
      <Show when={query() || group() || source()}><button type="button" class="btn btn--ghost" onClick={() => { setQuery(''); setGroup(''); setSource(''); setLimit(50); }}>{t('admin-config-clear')}</button></Show>
    </div></AdminSubsection>
    <Show when={busy()}><AsyncPending variant="status" /></Show>
    <Show when={!busy()}><p role="status">{t('admin-config-count', { count: filtered().length })}</p></Show>
    <Show when={!busy() && !error() && !unavailable() && filtered().length === 0}><p class={ux.pageState} role="status">{t('admin-config-no-results')}</p></Show>
    <Show when={filtered().length > 0}><AdminSubsection title={t('admin-config-entries')}>
      <div class={`${css.tableWrap} ${layout.scroller}`} tabindex="0" role="region" aria-label={t('admin-config-table')} aria-busy={busy()}><table class={`${css.table} ${layout.table}`}>
        <caption>{t('admin-config-table')}</caption>
        <thead><tr><th scope="col">{t('admin-config-key')}</th><th scope="col">{t('admin-config-value')}</th><th scope="col">{t('admin-config-source')}</th><th scope="col">{t('admin-config-change')}</th></tr></thead>
        <tbody><For each={filtered().slice(0, limit())}>{(row) => <tr>
          <th scope="row"><div class={layout.cell}><strong>{settingLabel(row.id) !== row.id ? settingLabel(row.id) : row.what ?? row.id}</strong><Show when={settingLabel(row.id) !== row.id || row.what}><code class={layout.key}>{row.id}</code></Show></div></th>
          <td><div class={layout.cell}><Show when={row.valueStatus !== 'unavailable'} fallback={<strong>{t('admin-config-unknown')}</strong>}><pre class={layout.value} dir="auto">{displayValue(row.value)}</pre></Show>
            <Show when={row.configuredInput !== undefined && row.configuredInput !== null}><details class={layout.details}><summary>{t('admin-config-input')}</summary><pre class={layout.value} dir="auto">{displayValue(row.configuredInput)}</pre><p class={css.note}>{t('admin-config-input-note')}</p></details></Show></div>
          </td>
          <td><div class={layout.cell}><span class={css.badge}>{t(`admin-config-source-${row.source}`)}</span><Show when={row.envVar}><code class={layout.key}>{row.envVar}</code></Show><Show when={row.locked}><span><Icon name="lock" /> {t('admin-config-locked')}</span></Show></div></td>
          <td><p>{row.restartNeeded ? t('admin-config-restart') : t('admin-config-live')}</p><Show when={row.configuredBy}><p>{row.configuredBy}</p></Show>
            <Show when={row.href?.startsWith('#/settings?')}><a href={hrefFor(sectionPath('settings', [row.id]))}>{t('admin-config-open-setting')}</a></Show>
          </td>
        </tr>}</For></tbody>
      </table></div>
      <Show when={filtered().length > limit()}><button type="button" class="btn" onClick={() => setLimit((n) => n + 50)}>{t('admin-config-more')}</button></Show>
    </AdminSubsection></Show>
  </section>;
}
