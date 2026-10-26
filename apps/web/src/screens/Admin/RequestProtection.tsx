import { createSignal, onMount, Show, For, useContext, type JSX } from 'solid-js';
import { protectionApi, type ProtectionApi, type ProtectionSubject, type ProtectionEvent, type ProtectionStatus } from '../../api/admin/requestProtection.ts';
import { loadCatalog, t } from '../../i18n';
import { formatDateTime } from '../../i18n/format.ts';
import { ShellContext } from './shell/context.ts';
import { sectionPath } from './shell/router.ts';
import { DataTable, type Column } from './shell/DataTable.tsx';
import { ConfirmDialog } from './shell/ConfirmDialog.tsx';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import { KNOWN_KEYS, keySlug } from './Settings/keys.ts';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';
export function RequestProtection(props:{api?:ProtectionApi}):JSX.Element {
 void loadCatalog('admin-settings');
 const shell=useContext(ShellContext);const api=()=>props.api??protectionApi;
 const [status,setStatus]=createSignal<ProtectionStatus>();const [subjects,setSubjects]=createSignal<ProtectionSubject[]>([]);const [events,setEvents]=createSignal<ProtectionEvent[]>([]);
 const [subjectCursor,setSubjectCursor]=createSignal<string|null>(null);const [eventCursor,setEventCursor]=createSignal<string|null>(null);
 const [loading,setLoading]=createSignal(true);const [error,setError]=createSignal('');const [success,setSuccess]=createSignal('');const [selected,setSelected]=createSignal<ProtectionSubject>();
 const date=(seconds:number):string=>formatDateTime(new Date(seconds*1000).toISOString());
 const identity=(row:{subjectKind:string;subjectKey:string}):string=>`${t(`admin-protection-kind-${row.subjectKind}`)} · ${row.subjectKey.slice(0,12)}`;
 async function reload():Promise<void>{setLoading(true);setError('');try{const [current,rows,history]=await Promise.all([api().status(),api().subjects(),api().events()]);setStatus(current);setSubjects(rows.subjects);setSubjectCursor(rows.cursor);setEvents(history.events);setEventCursor(history.cursor);}catch(cause){setError(describeAdminError(cause));}finally{setLoading(false);}}
 onMount(()=>void reload());
 async function next(kind:'subjects'|'events'):Promise<void>{setLoading(true);setError('');try{if(kind==='subjects'){const cursor=subjectCursor();if(!cursor)return;const result=await api().subjects(cursor);setSubjects(result.subjects);setSubjectCursor(result.cursor);}else{const cursor=eventCursor();if(!cursor)return;const result=await api().events(cursor);setEvents(result.events);setEventCursor(result.cursor);}}catch(cause){setError(describeAdminError(cause));}finally{setLoading(false);}}
 const subjectColumns=():Column<ProtectionSubject>[]=>[
 {id:'subject',header:t('admin-protection-subject'),cell:identity},
 {id:'short',header:t('admin-protection-short'),cell:row=>`${row.snapshot.short.anomalous} / ${row.snapshot.short.normal}`},
 {id:'long',header:t('admin-protection-long'),cell:row=>`${row.snapshot.long.anomalous} / ${row.snapshot.long.normal}`},
 {id:'until',header:t('admin-protection-action-until'),cell:row=>row.snapshot.activeUntil>Date.now()/1000?date(row.snapshot.activeUntil):t('admin-protection-monitoring')},
 {id:'updated',header:t('admin-protection-observed-at'),cell:row=>date(row.updatedAt)},
 ];
 const eventColumns=():Column<ProtectionEvent>[]=>[
 {id:'at',header:t('admin-protection-observed-at'),cell:row=>date(row.observedAt)},
 {id:'subject',header:t('admin-protection-subject'),cell:identity},
 {id:'reason',header:t('admin-protection-reason'),cell:row=>t(`admin-protection-reason-${row.reason}`)},
 {id:'action',header:t('admin-protection-action'),cell:row=>`${t(`admin-protection-action-${row.action}`)} · ${t(row.enforced?'admin-protection-enforced':'admin-protection-observed')}`},
 {id:'evidence',header:t('admin-protection-evidence'),cell:row=><details><summary>{t('admin-protection-evidence')}</summary><p class={css.note}>{t('admin-protection-evidence-counts',{shortBad:row.snapshot.short.anomalous,shortNormal:row.snapshot.short.normal,longBad:row.snapshot.long.anomalous,longNormal:row.snapshot.long.normal})}</p><Show when={row.policy}>{policy=><p class={css.note}>{t('admin-protection-evidence-thresholds',{shortSecs:policy().short.seconds,shortAbsolute:policy().short.absolute_anomalies,shortRatio:policy().short.ratio_percent,shortMinimum:policy().short.minimum_samples,longSecs:policy().long.seconds,longAbsolute:policy().long.absolute_anomalies,longRatio:policy().long.ratio_percent,longMinimum:policy().long.minimum_samples})}</p>}</Show><p class={css.note}>{row.snapshot.lastSignal??t('admin-protection-no-signal')}</p></details>},
 ];
 return <section class={css.section} aria-label={t('admin-protection-title')}>
 <div class={ux.pageHead}><h2 class={css.heading}>{t('admin-protection-title')}</h2><div class={ux.pageActions}><button class="btn btn--ghost" onClick={()=>void shell?.router.navigate(sectionPath('security'))}>{t('admin-security-center-title')}</button><button class="btn" disabled={loading()} onClick={()=>void reload()}>{t('admin-protection-refresh')}</button></div></div>
 <p class={css.note}>{t('admin-protection-explanation')}</p><Feedback kind="success" as="toast" message={success()}/>
 <Show when={status()}>{current=><div class={css.card}><h3 class={css.heading}>{t('admin-protection-current-policy')}</h3><p>{t('admin-protection-modes',{rate:t(`admin-protection-mode-${current().policy.rateMode}`),adaptive:t(`admin-protection-mode-${current().policy.adaptiveMode}`)})}</p><p class={css.note}>{t('admin-protection-retention',{subjects:current().subjectCapacity,events:current().eventCapacity,days:current().eventRetentionSecs/86400,seconds:current().propagationSecs})}</p><p class={css.note}>{t('admin-protection-coverage')}</p></div>}</Show>
 <Show when={shell?.can.can('settings.read')}><details class={css.card}><summary>{t('admin-protection-configure')}</summary><div class={css.grid}><For each={KNOWN_KEYS.filter(key=>key.startsWith('security.protection.'))}>{key=><button class="btn btn--ghost" onClick={()=>void shell?.router.navigate(sectionPath('settings',[key]))}>{t(`admin-settings-key-${keySlug(key)}`)}</button>}</For></div></details></Show>
 <div class={css.card}><h3 class={css.heading}>{t('admin-protection-subjects')}</h3><p class={css.note}>{t('admin-protection-counts-note')}</p><DataTable caption={t('admin-protection-subjects')} columns={subjectColumns()} rows={subjects()} rowKey={row=>row.subjectKey} state={loading()?'loading':error()?'error':'ready'} errorText={error()} onRetry={()=>void reload()} emptyText={t('admin-protection-empty')} rowActions={row=><Show when={shell?.can.can('bans.write')}><button class="btn btn--ghost btn--sm" aria-label={t('admin-protection-clear-target',{subject:identity(row)})} onClick={()=>setSelected(row)}>{t('admin-protection-clear')}</button></Show>}/><Show when={subjectCursor()}><button class="btn" disabled={loading()} onClick={()=>void next('subjects')}>{t('admin-protection-next')}</button></Show></div>
 <div class={css.card}><h3 class={css.heading}>{t('admin-protection-history')}</h3><DataTable caption={t('admin-protection-history')} columns={eventColumns()} rows={events()} rowKey={row=>row.id} state={loading()?'loading':error()?'error':'ready'} errorText={error()} onRetry={()=>void reload()} emptyText={t('admin-protection-history-empty')}/><Show when={eventCursor()}><button class="btn" disabled={loading()} onClick={()=>void next('events')}>{t('admin-protection-next')}</button></Show></div>
 <ConfirmDialog open={selected()!==undefined} title={t('admin-protection-clear-target',{subject:selected()?identity(selected()!):''})} body={t('admin-protection-clear-body')} confirmLabel={t('admin-protection-clear')} onCancel={()=>setSelected(undefined)} onConfirm={async()=>{const subject=selected();if(!subject)return;await api().clear(subject);setSelected(undefined);setSuccess(t('admin-protection-cleared',{subject:identity(subject)}));await reload();}}/>
 </section>;
}
