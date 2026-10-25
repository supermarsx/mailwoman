import { For, Show, createSignal, onMount, type JSX } from 'solid-js';
import { t } from '../../i18n';
import { adminRead, type AdminReadResult } from './shell/adminApi.ts';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';
export interface ApplicationEvent {
    id: number;
    firstSeen: number;
    lastSeen: number;
    repeatCount: number;
    level: string;
    target: string;
    fields: Record<string, unknown>;
}
export interface ApplicationHistory {
    events: ApplicationEvent[];
    cursor: number | null;
    discarded: number;
    capacity: number;
    retentionSeconds: number;
    processLocal: boolean;
}
export interface ApplicationLogsApi {
    history: (params: URLSearchParams) => Promise<AdminReadResult<ApplicationHistory>>;
}
export function ApplicationLogs(props: {
    api?: ApplicationLogsApi;
}): JSX.Element {
    const [rows, setRows] = createSignal<ApplicationEvent[]>([]);
    const [cursor, setCursor] = createSignal<number | null>(null);
    const [busy, setBusy] = createSignal(false);
    const [error, setError] = createSignal<string>();
    const [notice, setNotice] = createSignal<string>();
    const [discarded, setDiscarded] = createSignal(0);
    const [query, setQuery] = createSignal('');
    const [target, setTarget] = createSignal('');
    const [level, setLevel] = createSignal('');
    let applied = new URLSearchParams();
    async function load(more = false): Promise<void> {
        if (busy())
            return;
        setBusy(true);
        setError(undefined);
        setNotice(undefined);
        if (!more) {
            applied = new URLSearchParams();
            if (query())
                applied.set('q', query());
            if (target())
                applied.set('target', target());
            if (level())
                applied.set('level', level());
        }
        const params = new URLSearchParams(applied);
        if (more && cursor() != null)
            params.set('cursor', String(cursor()));
        try {
            const response = await (props.api?.history(params) ?? adminRead<ApplicationHistory>(`/logs/application?${params}`));
            if (response.kind === 'unavailable') {
                setNotice(t('admin-app-logs-unavailable'));
                return;
            }
            setRows(more ? [...rows(), ...response.data.events] : response.data.events);
            setCursor(response.data.cursor);
            setDiscarded(response.data.discarded);
        }
        catch (e) {
            setError(describeAdminError(e));
        }
        finally {
            setBusy(false);
        }
    }
    onMount(() => void load());
    return <section class={css.section} aria-label={t('admin-app-logs-title')}><h2 class={css.heading}>{t('admin-app-logs-title')}</h2><p class={css.note}>{t('admin-app-logs-description')}</p><p class={css.note}>{t('admin-app-logs-discarded', { count: discarded() })}</p>
 <form class={css.grid} onSubmit={(e) => { e.preventDefault(); void load(); }}>
 <label class="field-label">{t('admin-app-logs-search')}<input class="field" value={query()} maxlength={256} onInput={(e) => setQuery(e.currentTarget.value)}/></label>
 <label class="field-label">{t('admin-app-logs-target')}<input class="field" value={target()} maxlength={128} onInput={(e) => setTarget(e.currentTarget.value)}/></label>
 <label class="field-label">{t('admin-app-logs-level')}<select class="field" value={level()} onChange={(e) => setLevel(e.currentTarget.value)}><option value="">{t('admin-auth-all')}</option><option value="WARN">WARN</option><option value="ERROR">ERROR</option></select></label>
 <div class={ux.pageActions}><button class="btn" type="submit" disabled={busy()}>{t('admin-auth-apply')}</button></div></form>
 <Show when={busy()}><p role="status">{t('common-loading')}</p></Show><Feedback kind="error" message={error()}/><Feedback kind="info" message={notice()}/>
 <div class={css.tableWrap}><table class={css.table}><caption>{t('admin-app-logs-title')}</caption><thead><tr><th>{t('admin-auth-time')}</th><th>{t('admin-app-logs-level')}</th><th>{t('admin-app-logs-target')}</th><th>{t('admin-app-logs-details')}</th><th>{t('admin-auth-count')}</th></tr></thead><tbody><For each={rows()}>{(row) => <tr><td>{new Date(row.firstSeen * 1000).toLocaleString()}<Show when={row.lastSeen !== row.firstSeen}><br />{t('admin-auth-last')} {new Date(row.lastSeen * 1000).toLocaleString()}</Show></td><td>{row.level}</td><th scope="row">{row.target}</th><td><For each={Object.entries(row.fields)}>{([key, value]) => <p><strong>{key}: </strong>{typeof value === 'string' ? value : JSON.stringify(value)}</p>}</For></td><td>{row.repeatCount}</td></tr>}</For></tbody></table></div>
 <Show when={!busy() && !rows().length && !notice()}><p>{t('admin-app-logs-empty')}</p></Show><Show when={cursor() != null}><button class="btn" disabled={busy()} onClick={() => void load(true)}>{t('admin-auth-more')}</button></Show></section>;
}
