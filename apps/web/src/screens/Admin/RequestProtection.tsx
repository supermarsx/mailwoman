import { createEffect, createSignal, onMount, Show, For, useContext, type JSX } from 'solid-js';
import { protectionApi, type ProtectionApi, type ProtectionSubject, type ProtectionEvent, type ProtectionStatus } from '../../api/admin/requestProtection.ts';
import { loadCatalog, t } from '../../i18n';
import { formatDateTime, formatLocale } from '../../i18n/format.ts';
import { ShellContext } from './shell/context.ts';
import { sectionPath } from './shell/router.ts';
import { DataTable, type Column } from './shell/DataTable.tsx';
import { SectionTabs } from './shell/SectionTabs.tsx';
import { ConfirmDialog } from './shell/ConfirmDialog.tsx';
import { Feedback, describeAdminError } from './shell/Feedback.tsx';
import { KNOWN_KEYS, keySlug } from './Settings/keys.ts';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';
import { ProtectionPolicyEditor } from './ProtectionPolicyEditor.tsx';
import {Icon} from '../../components/Icon.tsx';
import { AdminProbeCard } from './AdminProbeCard.tsx';
import { SessionWatchCard } from './SessionWatchCard.tsx';
import type { AdminProbeApi } from '../../api/admin/adminProbe.ts';
import { formatSeconds } from './protectionRules.ts';
import { createPager, PAGE_SIZES } from './protectionPager.ts';
import type { ProtectionPolicyApi } from '../../api/admin/requestProtection.ts';

/** The screen's subtabs (`#/security/protection/<id>`); an unknown id opens the Overview. */
export const PROTECTION_TABS = ['overview', 'aggregate', 'adaptive', 'state', 'decisions'] as const;
export type ProtectionTab = (typeof PROTECTION_TABS)[number];
export function protectionTab(rest: readonly string[]): ProtectionTab {
 const id = rest[1] ?? '';
 return (PROTECTION_TABS as readonly string[]).includes(id) ? (id as ProtectionTab) : 'overview';
}

export function RequestProtection(props:{api?:ProtectionApi;policyApi?:ProtectionPolicyApi;probeApi?:AdminProbeApi}):JSX.Element {
 void loadCatalog('admin-settings');void loadCatalog('admin-security');
 const shell=useContext(ShellContext);const api=()=>props.api??protectionApi;
 const [status,setStatus]=createSignal<ProtectionStatus>();
 const subjectPager=createPager<ProtectionSubject>(async(after,limit)=>{const r=await api().subjects(after,limit);return{items:r.subjects,cursor:r.cursor,total:r.total??r.subjects.length};});
 const eventPager=createPager<ProtectionEvent>(async(after,limit)=>{const r=await api().events(after,limit);return{items:r.events,cursor:r.cursor,total:r.total??r.events.length};});
 const tab=():ProtectionTab=>protectionTab(shell?.router.route().rest??[]);
 const [loading,setLoading]=createSignal(true);const [error,setError]=createSignal('');const [success,setSuccess]=createSignal('');const [selected,setSelected]=createSignal<ProtectionSubject>();
 const canWrite=():boolean=>!shell||shell.can.can('settings.write');
 const date=(seconds:number):string=>formatDateTime(new Date(seconds*1000).toISOString());
 const identity=(row:{subjectKind:string;subjectKey:string}):string=>`${t(`admin-protection-kind-${row.subjectKind}`)} · ${row.subjectKey.slice(0,12)}`;
 const country=(row:{subjectKind:string;snapshot:{countryCode?:string|null}}):string=>{
  if(row.subjectKind!=='client'&&!row.subjectKind.startsWith('ua-'))return t('admin-protection-country-not-applicable');
  const code=row.snapshot.countryCode;
  if(typeof code!=='string'||!(/^[A-Z]{2}$/).test(code))return t('admin-protection-country-unknown');
  try{return new Intl.DisplayNames(formatLocale(),{type:'region',fallback:'none'}).of(code)??t('admin-protection-country-unknown');}catch{return code;}
 };
 /** Refresh the status line and whichever list is on screen; the other list loads when its subtab opens. */
 async function reload():Promise<void>{
  setLoading(true);setError('');
  try{setStatus(await api().status());}catch(cause){setError(describeAdminError(cause));}finally{setLoading(false);}
  if(tab()==='state')await subjectPager.reload();
  if(tab()==='decisions')await eventPager.reload();
 }
 onMount(()=>void reload());
 createEffect(()=>{
  const current=tab();
  if(current==='state'&&!subjectPager.loaded()&&!subjectPager.loading())void subjectPager.reload();
  if(current==='decisions'&&!eventPager.loaded()&&!eventPager.loading())void eventPager.reload();
 });
 /** Page-size picker and "page X of Y · N total" for one newest-first list. */
 const pagerBar=(pager:{page:()=>number;pages:()=>number;total:()=>number;size:()=>number;setSize:(n:number)=>Promise<void>},label:string):JSX.Element=>
  <div class={css.noteRow} data-testid="protection-pager">
   <span role="status">{t('admin-protection-page-status',{page:pager.page(),pages:pager.pages(),total:pager.total()})}</span>
   <label>{t('admin-protection-page-size')} <select aria-label={`${label} · ${t('admin-protection-page-size')}`} value={String(pager.size())} onChange={event=>void pager.setSize(Number(event.currentTarget.value))}><For each={PAGE_SIZES}>{size=><option value={String(size)} selected={pager.size()===size}>{size}</option>}</For></select></label>
  </div>;
 const subjectColumns=():Column<ProtectionSubject>[]=>[
 {id:'subject',header:t('admin-protection-subject'),cell:identity},
 {id:'ip',header:t('admin-protection-last-source-ip'),cell:row=>row.snapshot.sourceIp||t('admin-protection-ip-unknown')},
 {id:'country',header:t('admin-protection-country'),cell:country},
 {id:'short',header:t('admin-protection-short'),cell:row=>`${row.snapshot.short.anomalous} / ${row.snapshot.short.normal}`},
 {id:'long',header:t('admin-protection-long'),cell:row=>`${row.snapshot.long.anomalous} / ${row.snapshot.long.normal}`},
 {id:'until',header:t('admin-protection-action-until'),cell:row=>row.snapshot.activeUntil>Date.now()/1000?date(row.snapshot.activeUntil):t('admin-protection-monitoring')},
 {id:'updated',header:t('admin-protection-observed-at'),cell:row=>date(row.updatedAt)},
 ];
 /** A recent decision in words (t97): what happened, to whom, and which rule. */
 const decision=(row:ProtectionEvent):string=>{
  if(row.action==='cleared')return t('admin-protection-reason-manual-clear');
  const windowCounts=row.reason.startsWith('long')?row.snapshot.long:row.snapshot.short;
  const windowSecs=row.reason.startsWith('long')?row.snapshot.longWindowSecs:row.snapshot.shortWindowSecs;
  return t(`admin-protect-decision-${row.reason}`,{bad:windowCounts.anomalous,total:windowCounts.anomalous+windowCounts.normal,window:formatSeconds(windowSecs||row.snapshot.rateWindowSecs||60),requests:row.snapshot.rateCount,duration:formatSeconds(Math.max(1,row.retryAfterSecs)),signal:row.snapshot.lastSignal?t(`admin-protect-signal-${row.snapshot.lastSignal}`):t('admin-protect-none')});
 };
 const eventColumns=():Column<ProtectionEvent>[]=>[
 {id:'at',header:t('admin-protection-observed-at'),cell:row=>date(row.observedAt)},
 {id:'subject',header:t('admin-protection-subject'),cell:identity},
 {id:'ip',header:t('admin-protection-source-ip'),cell:row=>row.snapshot.sourceIp||t('admin-protection-ip-unknown')},
 {id:'country',header:t('admin-protection-country'),cell:country},
 {id:'reason',header:t('admin-protection-reason'),cell:row=><span data-testid="protection-decision">{decision(row)}</span>},
 {id:'action',header:t('admin-protection-action'),cell:row=>`${t(`admin-protection-action-${row.action}`)} · ${t(row.enforced?'admin-protection-enforced':'admin-protection-observed')}`},
 {id:'evidence',header:t('admin-protection-evidence'),cell:row=><details><summary>{t('admin-protection-evidence')}</summary><p class={css.note}>{t('admin-protection-evidence-counts',{shortBad:row.snapshot.short.anomalous,shortNormal:row.snapshot.short.normal,longBad:row.snapshot.long.anomalous,longNormal:row.snapshot.long.normal})}</p><Show when={row.policy}>{policy=><p class={css.note}>{t('admin-protection-evidence-thresholds',{shortSecs:policy().short.seconds,shortAbsolute:policy().short.absolute_anomalies,shortRatio:policy().short.ratio_percent,shortMinimum:policy().short.minimum_samples,longSecs:policy().long.seconds,longAbsolute:policy().long.absolute_anomalies,longRatio:policy().long.ratio_percent,longMinimum:policy().long.minimum_samples})}</p>}</Show><p class={css.note}>{row.snapshot.lastSignal??t('admin-protection-no-signal')}</p></details>},
 ];
 return <section class={css.section} aria-label={t('admin-protection-title')}>
 <div class={ux.pageHead}><h2 class={css.heading}>{t('admin-protection-title')}</h2><div class={ux.pageActions}><button class="btn btn--ghost" onClick={()=>void shell?.router.navigate(sectionPath('security'))}>{t('admin-security-center-title')}</button><button class="btn" disabled={loading()||subjectPager.loading()||eventPager.loading()} onClick={()=>void reload()}>{t('admin-protection-refresh')}</button></div></div>
 <p class={css.note}>{t('admin-protection-explanation')}</p><p class={css.note}>{t('admin-protect-not-country')}</p><Feedback kind="success" as="toast" message={success()}/>
 <SectionTabs label={t('admin-protection-tabs')} items={PROTECTION_TABS.map(id=>({id,label:t(`admin-protection-tab-${id}`),path:sectionPath('security',['protection',id])}))} selected={tab()} navigate={path=>void shell?.router.navigate(path)}/>
 <Show when={tab()==='overview'}>
  <Show when={status()}>{current=><div class={css.card}><h3 class={css.heading}>{t('admin-protection-current-policy')}</h3><p>{t('admin-protection-modes',{rate:t(`admin-protection-mode-${current().policy.rateMode}`),adaptive:t(`admin-protection-mode-${current().policy.adaptiveMode}`)})}</p><p class={css.note}>{t('admin-protection-retention',{subjects:current().subjectCapacity,events:current().eventCapacity,days:current().eventRetentionSecs/86400,seconds:current().propagationSecs})}</p><p class={css.note}>{t('admin-protection-coverage')}</p></div>}</Show>
  <Show when={status()?.protocolGuard}>{guard=><div class={css.card} data-testid="protocol-guard"><h3 class={css.heading}>{t('admin-protocol-title')}</h3><p class={css.note}>{t('admin-protocol-note')}</p><table><thead><tr><th>{t('admin-protocol-class')}</th><th>{t('admin-protocol-mode')}</th><th>{t('admin-protocol-detected')}</th><th>{t('admin-protocol-enforced')}</th><th>{t('admin-protocol-last-seen')}</th></tr></thead><tbody><For each={guard().classes}>{row=><tr><td>{t(`admin-settings-key-${keySlug(`security.protocol.${row.class}Mode`)}`)}</td><td>{t(`admin-protocol-mode-${row.mode}`)}</td><td>{row.detected}</td><td>{row.enforced}</td><td>{row.lastSeen?date(row.lastSeen):t('admin-protocol-never')}</td></tr>}</For></tbody></table><Show when={shell?.can.can('settings.read')}><div class={css.grid}><For each={KNOWN_KEYS.filter(key=>key.startsWith('security.protocol.'))}>{key=><button class="btn btn--ghost" onClick={()=>void shell?.router.navigate(sectionPath('settings',[key]))}>{t(`admin-settings-key-${keySlug(key)}`)}</button>}</For></div></Show></div>}</Show>
  <AdminProbeCard api={props.probeApi}/>
  <SessionWatchCard/>
  <ProtectionPolicyEditor part="overview" api={props.policyApi} canWrite={canWrite()} onSaved={()=>void reload()}/>
 </Show>
 <Show when={tab()==='aggregate'}><p class={css.note}>{t('admin-protection-aggregate-help')}</p><ProtectionPolicyEditor part="aggregate" api={props.policyApi} canWrite={canWrite()} onSaved={()=>void reload()}/></Show>
 <Show when={tab()==='adaptive'}><p class={css.note}>{t('admin-protection-adaptive-help')}</p><ProtectionPolicyEditor part="adaptive" api={props.policyApi} canWrite={canWrite()} onSaved={()=>void reload()}/></Show>
 <Show when={tab()==='state'}><div class={css.card}><h3 class={css.heading}>{t('admin-protection-subjects')}</h3><p class={css.note}>{t('admin-protection-counts-note')}</p><p class={css.note}>{t('admin-protection-newest-first')}</p>{pagerBar(subjectPager,t('admin-protection-subjects'))}<DataTable caption={t('admin-protection-subjects')} columns={subjectColumns()} rows={subjectPager.items()} rowKey={row=>row.subjectKey} state={subjectPager.loading()?'loading':(subjectPager.error()||error())?'error':'ready'} errorText={subjectPager.error()||error()} onRetry={()=>void reload()} emptyText={t('admin-protection-empty')} paging={{hasPrev:subjectPager.hasPrev(),hasNext:subjectPager.hasNext(),onPrev:()=>void subjectPager.prevPage(),onNext:()=>void subjectPager.nextPage()}} rowActions={row=><Show when={shell?.can.can('bans.write')}><button type="button" class={`btn btn--ghost ${css.compactIconAction}`} aria-label={t('admin-protection-clear-target',{subject:identity(row)})} data-tooltip={t('admin-protection-clear')} title={t('admin-protection-clear')} onClick={()=>setSelected(row)}><Icon name="unlock"/></button></Show>}/></div></Show>
 <Show when={tab()==='decisions'}><div class={css.card}><h3 class={css.heading}>{t('admin-protection-history')}</h3><p class={css.note}>{t('admin-protection-newest-first')}</p>{pagerBar(eventPager,t('admin-protection-history'))}<DataTable caption={t('admin-protection-history')} columns={eventColumns()} rows={eventPager.items()} rowKey={row=>row.id} state={eventPager.loading()?'loading':(eventPager.error()||error())?'error':'ready'} errorText={eventPager.error()||error()} onRetry={()=>void reload()} emptyText={t('admin-protection-history-empty')} paging={{hasPrev:eventPager.hasPrev(),hasNext:eventPager.hasNext(),onPrev:()=>void eventPager.prevPage(),onNext:()=>void eventPager.nextPage()}}/></div></Show>
 <ConfirmDialog open={selected()!==undefined} title={t('admin-protection-clear-target',{subject:selected()?identity(selected()!):''})} body={t('admin-protection-clear-body')} confirmLabel={t('admin-protection-clear')} onCancel={()=>setSelected(undefined)} onConfirm={async()=>{const subject=selected();if(!subject)return;await api().clear(subject);setSelected(undefined);setSuccess(t('admin-protection-cleared',{subject:identity(subject)}));await reload();}}/>
 </section>;
}
