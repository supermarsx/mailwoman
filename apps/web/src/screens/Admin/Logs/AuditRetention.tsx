// Admin › Observability › Audit log: how long audit entries are kept (t39 L5).
//
// One row per category: the default (`*`) and administrator actions, security
// events and user alerts. A category without its own value follows the default;
// with no default every entry is kept. A legal hold stops pruning for its
// category (on the default: for every category). Saving a number of days the
// first time creates the daily prune task; "Prune now" applies the policy at
// once. Both are recorded in the audit log.

import { For, Show, createSignal, createUniqueId, onMount, useContext, type JSX } from 'solid-js';
import { formatDateTime, loadCatalog, t } from '../../../i18n';
import { AdminProblemError, adminRead, adminRequest } from '../shell/adminApi.ts';
import { ConfirmDialog } from '../shell/ConfirmDialog.tsx';
import { Feedback } from '../shell/Feedback.tsx';
import { clearAdminError, showAdminError } from '../shell/requestErrorToast.ts';
import { ShellContext } from '../shell/context.ts';
import * as css from '../admin.css.ts';
import * as shellCss from '../shell/shell.css.ts';
import * as ux from '../shell/ux.css.ts';

export interface RetentionRow {
  category: string;
  days: number | null;
  legalHold: boolean;
  updatedBy: string;
  updatedAt: number;
}

export interface RetentionState {
  categories: string[];
  rows: RetentionRow[];
  bounds: { minDays: number; maxDays: number };
  lastPrune: { ts: number } | null;
  task: { id: string; enabled: boolean; nextRunAt: number | null } | null;
  report?: { total: number; removed: Record<string, number> };
}

export interface RetentionApi {
  load(): Promise<RetentionState | null>;
  put(category: string, body: { days: number | null; legalHold: boolean }): Promise<RetentionState>;
  reset(category: string): Promise<RetentionState>;
  prune(): Promise<RetentionState>;
}

const enc = encodeURIComponent;
export const retentionApi: RetentionApi = {
  async load() {
    const r = await adminRead<RetentionState>('/audit/retention');
    return r.kind === 'ok' ? r.data : null;
  },
  async put(category, body) {
    return (await adminRequest<RetentionState>(`/audit/retention/${enc(category)}`, { method: 'PUT', body })).data;
  },
  async reset(category) {
    return (await adminRequest<RetentionState>(`/audit/retention/${enc(category)}`, { method: 'DELETE' })).data;
  },
  async prune() {
    return (await adminRequest<RetentionState>('/audit/retention/prune', { method: 'POST' })).data;
  },
};

function categoryLabel(c: string): string {
  switch (c) {
    case '*': return t('admin-retention-cat-default');
    case 'admin': return t('admin-retention-cat-admin');
    case 'security': return t('admin-retention-cat-security');
    case 'alert': return t('admin-retention-cat-alert');
    default: return c;
  }
}

type Mode = 'inherit' | 'forever' | 'days';

function CategoryRow(props: {
  category: string;
  row: RetentionRow | undefined;
  defaults: RetentionRow | undefined;
  bounds: RetentionState['bounds'];
  canWrite: boolean;
  onSave: (body: { days: number | null; legalHold: boolean } | null) => Promise<void>;
}): JSX.Element {
  const isDefault = props.category === '*';
  const initialMode = (): Mode => (props.row ? (props.row.days === null ? 'forever' : 'days') : isDefault ? 'forever' : 'inherit');
  const [mode, setMode] = createSignal<Mode>(initialMode());
  const [days, setDays] = createSignal(props.row?.days ?? props.defaults?.days ?? 365);
  const [hold, setHold] = createSignal(props.row?.legalHold ?? false);
  const [busy, setBusy] = createSignal(false);
  const uid = createUniqueId();
  const label = (): string => categoryLabel(props.category);
  const outOfBounds = (): boolean => mode() === 'days' && (!Number.isInteger(days()) || days() < props.bounds.minDays || days() > props.bounds.maxDays);
  const current = (): string => {
    const r = props.row ?? (isDefault ? undefined : props.defaults);
    const kept = !r || r.days === null ? t('admin-retention-forever') : t('admin-retention-days', { days: r.days });
    return props.row || isDefault ? kept : t('admin-retention-follows-default', { kept });
  };
  async function save(e: SubmitEvent): Promise<void> {
    e.preventDefault();
    if (outOfBounds()) return;
    setBusy(true);
    try {
      await props.onSave(mode() === 'inherit' ? null : { days: mode() === 'days' ? days() : null, legalHold: hold() });
    } finally {
      setBusy(false);
    }
  }
  return (
    <tr data-category={props.category}>
      <th scope="row">{label()}</th>
      <td><span data-kept>{current()}</span><Show when={props.row?.legalHold}><br /><span class={css.badge} data-hold>{t('admin-retention-hold-on')}</span></Show></td>
      <td>
        <Show when={props.canWrite} fallback={<span class={css.note}>{t('admin-retention-read-only')}</span>}>
          <form class={ux.pageActions} onSubmit={(e) => void save(e)} aria-label={t('admin-retention-edit-for', { category: label() })}>
            <div class="field"><label for={`${uid}-mode`}>{t('admin-retention-keep')}</label>
              <select id={`${uid}-mode`} value={mode()} onChange={(e) => setMode(e.currentTarget.value as Mode)}>
                <Show when={!isDefault}><option value="inherit">{t('admin-retention-mode-inherit')}</option></Show>
                <option value="forever">{t('admin-retention-mode-forever')}</option>
                <option value="days">{t('admin-retention-mode-days')}</option>
              </select></div>
            <Show when={mode() === 'days'}>
              <div class="field"><label for={`${uid}-days`}>{t('admin-retention-days-label')}</label>
                <input id={`${uid}-days`} type="number" min={props.bounds.minDays} max={props.bounds.maxDays} step="1" value={days()}
                  aria-invalid={outOfBounds()} onInput={(e) => setDays(e.currentTarget.valueAsNumber)} /></div>
            </Show>
            <Show when={mode() !== 'inherit'}>
              <label class={ux.checkboxRow}><input type="checkbox" checked={hold()} onChange={(e) => setHold(e.currentTarget.checked)} /><span>{t('admin-retention-hold')}</span></label>
            </Show>
            <button type="submit" class="btn" disabled={busy() || outOfBounds()}>{t('admin-retention-save')}<span class={shellCss.srOnly}> {label()}</span></button>
            <Show when={outOfBounds()}><p class={css.error}>{t('admin-retention-bounds', { min: props.bounds.minDays, max: props.bounds.maxDays })}</p></Show>
          </form>
        </Show>
      </td>
    </tr>
  );
}

export function AuditRetention(props: { api?: RetentionApi; canWrite?: boolean; onChanged?: () => void }): JSX.Element {
  const api = props.api ?? retentionApi;
  const shell = useContext(ShellContext);
  const canWrite = (): boolean => props.canWrite ?? shell?.can.can('settings.write') ?? false;
  const [state, setState] = createSignal<RetentionState | null>();
  const [error, setError] = createSignal<string>();
  const [notice, setNotice] = createSignal<string>();
  const [confirming, setConfirming] = createSignal(false);
  // A domain-scoped auditor reads the log but not the deployment-wide policy.
  const [hidden, setHidden] = createSignal(false);
  async function load(): Promise<void> {
    try {
      setState(await api.load());
    } catch (e) {
      if (e instanceof AdminProblemError && e.status === 403) setHidden(true);
      else showAdminError(e, { key: 'audit-retention-load', retry: load }, setError);
    }
  }
  onMount(() => {
    void loadCatalog('admin-features');
    void load();
  });
  async function apply(fn: () => Promise<RetentionState>, done: (s: RetentionState) => string): Promise<void> {
    setError(undefined);
    setNotice(undefined);
    try {
      const s = await fn();
      setState(s);
      setNotice(done(s));
      props.onChanged?.();
      clearAdminError('audit-retention');
    } catch (e) {
      showAdminError(e, { key: 'audit-retention', retry: () => apply(fn, done) }, setError);
    }
  }
  const rowOf = (c: string): RetentionRow | undefined => state()?.rows.find((r) => r.category === c);
  const schedule = (): string => {
    const task = state()?.task;
    if (!task) return t('admin-retention-task-none');
    if (!task.enabled) return t('admin-retention-task-disabled');
    return task.nextRunAt ? t('admin-retention-task-next', { when: formatDateTime(new Date(task.nextRunAt * 1000), { dateStyle: 'medium', timeStyle: 'short' }) }) : t('admin-retention-task-daily');
  };
  return (
    <Show when={!hidden()}>
    <section class={css.section} aria-labelledby="audit-retention-title" data-testid="audit-retention">
      <h3 id="audit-retention-title" class={css.heading}>{t('admin-retention-title')}</h3>
      <p class={css.note}>{t('admin-retention-intro')}</p>
      <Feedback kind="error" message={error()} />
      <Feedback kind="success" message={notice()} testId="retention-notice" />
      <Show when={state() !== undefined} fallback={<p class={css.note}>{t('admin-retention-loading')}</p>}>
        <Show when={state()} fallback={<p class={css.note}>{t('admin-retention-unavailable')}</p>}>
          {(s) => (
            <>
              <div class={css.tableWrap} tabindex="0" role="group" aria-label={t('admin-retention-title')}>
                <table class={css.table}>
                  <caption class={shellCss.srOnly}>{t('admin-retention-title')}</caption>
                  <thead><tr>
                    <th scope="col">{t('admin-retention-col-category')}</th>
                    <th scope="col">{t('admin-retention-col-kept')}</th>
                    <th scope="col">{t('admin-retention-col-change')}</th>
                  </tr></thead>
                  <tbody>
                    <For each={s().categories}>
                      {(c) => (
                        <Show when={JSON.stringify(rowOf(c) ?? null) + JSON.stringify(rowOf('*') ?? null)} keyed>
                          {(_key: string) => (
                            <CategoryRow category={c} row={rowOf(c)} defaults={rowOf('*')} bounds={s().bounds} canWrite={canWrite()}
                              onSave={(body) => apply(() => (body ? api.put(c, body) : api.reset(c)), () => t('admin-retention-saved', { category: categoryLabel(c) }))} />
                          )}
                        </Show>
                      )}
                    </For>
                  </tbody>
                </table>
              </div>
              <p class={css.note}>{schedule()}</p>
              <p class={css.note}>{s().lastPrune ? t('admin-retention-last', { when: formatDateTime(new Date(s().lastPrune!.ts * 1000), { dateStyle: 'medium', timeStyle: 'short' }) }) : t('admin-retention-never')}</p>
              <Show when={canWrite()}>
                <div class={ux.pageActions}><button type="button" class="btn" onClick={() => setConfirming(true)}>{t('admin-retention-prune')}</button></div>
              </Show>
            </>
          )}
        </Show>
      </Show>
      <ConfirmDialog open={confirming()} title={t('admin-retention-prune-title')} body={t('admin-retention-prune-body')} confirmLabel={t('admin-retention-prune')}
        onCancel={() => setConfirming(false)}
        onConfirm={async () => {
          setConfirming(false);
          await apply(() => api.prune(), (r) => t('admin-retention-pruned', { count: r.report?.total ?? 0 }));
        }} />
    </section>
    </Show>
  );
}
