import {AsyncPending} from '../../components/AsyncState.tsx';
import { For, Show, createSignal, onMount, type JSX } from 'solid-js';
import { t, formatDateTime } from '../../i18n';
import { adminRead, type AdminReadResult } from './shell/adminApi.ts';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';
export interface AuthEvent {
    id: string;
    accountId: string | null;
    usernameAttempted: string;
    ts: number;
    lastSeen: number;
    repeatCount: number;
    result: string;
    method: string;
    ip: string;
    userAgent: string;
    reason: string;
    surface: string;
    backend: string | null;
}
export interface AuthHistory {
    events: AuthEvent[];
    cursor: string | null;
    retentionDays: number;
}
export interface AuthLogsApi {
    history: (params: URLSearchParams) => Promise<AdminReadResult<AuthHistory>>;
}
const httpApi: AuthLogsApi = { history: (params) => adminRead<AuthHistory>(`/logs/auth?${params}`) };
export function AuthLogs(props: {
    api?: AuthLogsApi;
}): JSX.Element {
    const [rows, setRows] = createSignal<AuthEvent[]>([]);
    const [cursor, setCursor] = createSignal<string | null>(null);
    const [busy, setBusy] = createSignal(false);
    const [error, setError] = createSignal<string>();
    const [notice, setNotice] = createSignal<string>();
    const [query, setQuery] = createSignal('');
    const [surface, setSurface] = createSignal('');
    const [result, setResult] = createSignal('');
    const [backend, setBackend] = createSignal('');
    const [reason, setReason] = createSignal('');
    const [days, setDays] = createSignal(7);
    let applied = new URLSearchParams();
    async function load(more = false): Promise<void> {
        if (busy())
            return;
        setBusy(true);
        setError(undefined);
        setNotice(undefined);
        if (!more) {
            applied = new URLSearchParams();
            for (const [key, value] of [['q', query()], ['surface', surface()], ['result', result()], ['backend', backend()], ['reason', reason()]])
                if (value)
                    applied.set(key!, value!);
        }
        const params = new URLSearchParams(applied);
        if (more && cursor())
            params.set('cursor', cursor()!);
        try {
            const response = await (props.api ?? httpApi).history(params);
            if (response.kind === 'unavailable') {
                setNotice(t('admin-auth-unavailable'));
                return;
            }
            setRows(more ? [...rows(), ...response.data.events] : response.data.events);
            setCursor(response.data.cursor);
            setDays(response.data.retentionDays);
        }
        catch (e) {
            setError(describeAdminError(e));
        }
        finally {
            setBusy(false);
        }
    }
    onMount(() => void load());
    const missing = (): string => t('admin-auth-unknown');
    return <section class={css.section} aria-label={t('admin-auth-title')}>
  <h2 class={css.heading}>{t('admin-auth-title')}</h2><p class={css.note}>{t('admin-auth-description', { days: days() })}</p>
  <form class={css.grid} onSubmit={(e) => { e.preventDefault(); void load(); }}>
   <label class="field">{t('admin-auth-search')}<input value={query()} onInput={(e) => setQuery(e.currentTarget.value)} maxlength={128}/></label>
   <label class="field">{t('admin-auth-surface')}<select value={surface()} onChange={(e) => setSurface(e.currentTarget.value)}><option value="">{t('admin-auth-all')}</option><option value="webmail">{t('admin-auth-webmail')}</option><option value="admin">{t('admin-auth-admin')}</option></select></label>
   <label class="field">{t('admin-auth-result')}<select value={result()} onChange={(e) => setResult(e.currentTarget.value)}><option value="">{t('admin-auth-all')}</option><option value="success">{t('admin-auth-success')}</option><option value="failure">{t('admin-auth-failure')}</option><option value="blocked">{t('admin-auth-blocked')}</option></select></label>
   <label class="field">{t('admin-auth-backend')}<input value={backend()} onInput={(e) => setBackend(e.currentTarget.value)} maxlength={128}/></label>
   <label class="field">{t('admin-auth-reason')}<input value={reason()} onInput={(e) => setReason(e.currentTarget.value)} maxlength={64}/></label>
   <div class={ux.pageActions}><button class="btn" type="submit" disabled={busy()}>{t('admin-auth-apply')}</button></div>
  </form>
  <Show when={busy()}><AsyncPending variant={rows().length ? 'status' : 'table'}/></Show><Feedback kind="error" message={error()}/><Feedback kind="info" message={notice()}/>
  <div class={css.tableWrap} tabindex="0" role="group" aria-label={t('admin-auth-title')}><table class={css.table}><caption>{t('admin-auth-title')}</caption><thead><tr><For each={['time', 'account', 'surface', 'backend', 'result', 'reason', 'method', 'ip', 'count']}>{(key) => <th scope="col">{t(`admin-auth-${key}`)}</th>}</For></tr></thead>
  <tbody><For each={rows()}>{(row) => <tr><td><time datetime={new Date(row.ts * 1000).toISOString()}>{formatDateTime(new Date(row.ts * 1000), { dateStyle: 'medium', timeStyle: 'medium' })}</time><Show when={row.lastSeen !== row.ts}><br /><small>{t('admin-auth-last')} {formatDateTime(new Date(row.lastSeen * 1000), { dateStyle: 'medium', timeStyle: 'medium' })}</small></Show></td><th scope="row">{row.usernameAttempted || missing()}<Show when={row.accountId}><br /><small>{row.accountId}</small></Show></th><td>{row.surface}</td><td>{row.backend || missing()}</td><td>{row.result}</td><td>{row.reason || '—'}</td><td>{row.method}</td><td>{row.ip || missing()}<Show when={row.userAgent}><details><summary>{t('admin-auth-device')}</summary>{row.userAgent}</details></Show></td><td>{row.repeatCount}</td></tr>}</For></tbody></table></div>
  <Show when={!busy() && !rows().length && !notice() && !error()}><p>{t('admin-auth-empty')}</p></Show>
  <Show when={cursor()}><button class="btn" type="button" disabled={busy()} onClick={() => void load(true)}>{t('admin-auth-more')}</button></Show>
 </section>;
}
