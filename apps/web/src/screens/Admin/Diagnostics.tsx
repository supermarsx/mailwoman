import { For, Show, createSignal, onMount, type JSX } from 'solid-js';
import { t } from '../../i18n';
import { adminRead, adminRequest, type AdminReadResult } from './shell/adminApi.ts';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import { useShell } from './shell/context.ts';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';

interface Check { id: string; status: string; durationMs: number; summary: string }
interface Run { id: string; suite: string; startedAt: number; status: string; results: Check[] }
interface History { runs: Run[]; nextCursor: string | null }
export interface DiagnosticsApi {
  history: (cursor?: string) => Promise<AdminReadResult<History>>;
  run: () => Promise<{ run: Run; historySaved: boolean; auditSaved: boolean; retentionApplied?: boolean }>;
}
const httpApi: DiagnosticsApi = {
  history: (cursor) => adminRead<History>(`/diag/runs${cursor ? `?cursor=${encodeURIComponent(cursor)}` : ''}`),
  run: async () => (await adminRequest<{ run: Run; historySaved: boolean; auditSaved: boolean; retentionApplied?: boolean }>('/diag/run', { method: 'POST', body: { suite: 'server' } })).data,
};

export function Diagnostics(props: { api?: DiagnosticsApi }): JSX.Element {
  const shell = useShell();
  const api = (): DiagnosticsApi => props.api ?? httpApi;
  const [runs, setRuns] = createSignal<Run[]>([]);
  const [selected, setSelected] = createSignal<Run>();
  const [cursor, setCursor] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string>();
  const [notice, setNotice] = createSignal<string>();
  async function history(more = false): Promise<void> {
    setBusy(true); setError(undefined); setNotice(undefined);
    try {
      const result = await api().history(more ? cursor() ?? undefined : undefined);
      if (result.kind === 'unavailable') { setNotice(t('admin-diag-unavailable')); return; }
      setRuns(more ? [...runs(), ...result.data.runs] : result.data.runs);
      setCursor(result.data.nextCursor);
      if (!selected()) setSelected(result.data.runs[0]);
    } catch (e) { setError(describeAdminError(e)); }
    finally { setBusy(false); }
  }
  async function run(): Promise<void> {
    setBusy(true); setError(undefined); setNotice(undefined);
    try {
      const result = await api().run();
      setSelected(result.run);
      if (result.historySaved) setRuns([result.run, ...runs().filter((r) => r.id !== result.run.id)]);
      if (!result.historySaved || !result.auditSaved) setNotice(t('admin-diag-not-saved'));
      else if (result.retentionApplied === false) setNotice(t('admin-diag-retention-failed'));
    } catch (e) { setError(describeAdminError(e)); }
    finally { setBusy(false); }
  }
  onMount(() => void history());
  return <section class={css.section} aria-label={t('admin-diag-title')}>
    <h2 class={css.heading}>{t('admin-diag-title')}</h2>
    <p class={css.note}>{t('admin-diag-description')}</p>
    <div class={ux.pageActions}>
      <button class="btn btn--primary" type="button" disabled={busy() || !shell.can.can('diagnostics.run')} onClick={() => void run()}>{t('admin-diag-run')}</button>
      <button class="btn" type="button" disabled={busy()} onClick={() => void history()}>{t('admin-diag-refresh')}</button>
    </div>
    <Show when={busy()}><p role="status">{t('common-loading')}</p></Show>
    <Feedback kind="error" message={error()} /><Feedback kind="info" message={notice()} />
    <Show when={selected()}>{(current) => <div class={css.card}>
      <h3>{t('admin-diag-result')} — {current().status}</h3>
      <div class={css.tableWrap}><table class={css.table}>
        <caption>{t('admin-diag-checks')}</caption>
        <thead><tr><th scope="col">{t('admin-diag-check')}</th><th scope="col">{t('admin-diag-status')}</th><th scope="col">{t('admin-diag-duration')}</th></tr></thead>
        <tbody><For each={current().results}>{(check) => <tr><th scope="row">{check.summary}</th><td>{check.status}</td><td>{check.durationMs} ms</td></tr>}</For></tbody>
      </table></div>
    </div>}</Show>
    <h3>{t('admin-diag-history')}</h3>
    <Show when={runs().length > 0} fallback={<p>{t('admin-diag-empty')}</p>}>
      <ul><For each={runs()}>{(entry) => <li><button type="button" class="btn" onClick={() => setSelected(entry)}>{new Date(entry.startedAt * 1000).toLocaleString()} · {entry.suite} · {entry.status}</button></li>}</For></ul>
    </Show>
    <Show when={cursor()}><button class="btn" type="button" disabled={busy()} onClick={() => void history(true)}>{t('admin-diag-more')}</button></Show>
  </section>;
}
