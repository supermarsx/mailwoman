import { For, Show, createMemo, createSignal, onCleanup, onMount, type JSX } from 'solid-js';
import { t } from '../../i18n';
import { loadEffectiveConfig, type AdminReadResult, type EffectiveEntry } from './shell/adminApi.ts';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';

export interface ConfigurationProps { load?: () => Promise<AdminReadResult<EffectiveEntry[]>> }

function displayValue(value: unknown): string {
  if (value === null || value === undefined) return t('admin-config-empty');
  if (typeof value === 'object' && 'set' in value) return t((value as { set: boolean }).set ? 'admin-config-secret-set' : 'admin-config-secret-unset');
  return typeof value === 'string' ? value : JSON.stringify(value);
}

/** A redacted view of the server's report; unavailable runtime values stay unknown. */
export function Configuration(props: ConfigurationProps): JSX.Element {
  const [rows, setRows] = createSignal<EffectiveEntry[]>([]);
  const [query, setQuery] = createSignal(new URLSearchParams(location.hash.split('?')[1] ?? '').get('field') ?? '');
  const [group, setGroup] = createSignal('');
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
  onMount(() => void reload());
  const groups = createMemo(() => [...new Set(rows().map((row) => row.group))].sort());
  const filtered = createMemo(() => {
    const needle = query().trim().toLowerCase();
    return rows().filter((row) => (!group() || row.group === group()) && `${row.id} ${row.what ?? ''} ${row.searchText ?? ''} ${row.envVar ?? ''}`.toLowerCase().includes(needle));
  });
  return <section class={css.section} aria-label={t('admin-config-title')}>
    <div class={ux.pageHead}>
      <div><h2 class={css.heading}>{t('admin-config-title')}</h2><p class={css.note}>{t('admin-config-description')}</p></div>
      <button type="button" class="btn" disabled={busy()} onClick={() => void reload()}>{t('admin-config-refresh')}</button>
    </div>
    <Feedback kind="error" message={error()} />
    <Show when={unavailable()}><p role="status">{t('admin-config-unavailable')}</p></Show>
    <div class={css.grid}>
      <label class="field"><span>{t('admin-config-search')}</span><input type="search" value={query()} onInput={(e) => { setQuery(e.currentTarget.value); setLimit(50); }} /></label>
      <label class="field"><span>{t('admin-config-group')}</span><select value={group()} onChange={(e) => { setGroup(e.currentTarget.value); setLimit(50); }}><option value="">{t('admin-config-all')}</option><For each={groups()}>{(g) => <option value={g}>{g}</option>}</For></select></label>
    </div>
    <p role="status">{busy() ? t('common-loading') : t('admin-config-count', { count: filtered().length })}</p>
    <Show when={!busy() && filtered().length > 0}>
      <div class={css.tableWrap}><table class={css.table}>
        <caption>{t('admin-config-table')}</caption>
        <thead><tr><th scope="col">{t('admin-config-key')}</th><th scope="col">{t('admin-config-value')}</th><th scope="col">{t('admin-config-source')}</th><th scope="col">{t('admin-config-change')}</th></tr></thead>
        <tbody><For each={filtered().slice(0, limit())}>{(row) => <tr>
          <th scope="row"><code>{row.id}</code><Show when={row.what}><p class={css.note}>{row.what}</p></Show></th>
          <td><Show when={row.valueStatus !== 'unavailable'} fallback={<strong>{t('admin-config-unknown')}</strong>}><span dir="auto">{displayValue(row.value)}</span></Show>
            <Show when={row.configuredInput !== undefined && row.configuredInput !== null}><details><summary>{t('admin-config-input')}</summary><p dir="auto">{displayValue(row.configuredInput)}</p><p class={css.note}>{t('admin-config-input-note')}</p></details></Show>
          </td>
          <td><span class={css.badge}>{t(`admin-config-source-${row.source}`)}</span><Show when={row.locked}><p>{t('admin-config-locked')}</p></Show></td>
          <td><p>{row.restartNeeded ? t('admin-config-restart') : t('admin-config-live')}</p><Show when={row.configuredBy}><p>{row.configuredBy}</p></Show>
            <Show when={row.href?.startsWith('#/settings?')}><a href={row.href}>{t('admin-config-open-setting')}</a></Show>
          </td>
        </tr>}</For></tbody>
      </table></div>
      <Show when={filtered().length > limit()}><button type="button" class="btn" onClick={() => setLimit((n) => n + 50)}>{t('admin-config-more')}</button></Show>
    </Show>
  </section>;
}
