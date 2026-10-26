import { createSignal,onMount,Show,useContext,type JSX } from 'solid-js';
import { loadSigninGeography,type SigninGeography } from '../../api/admin/signinGeography.ts';
import { t } from '../../i18n';
import { ShellContext } from './shell/context.ts';
import { sectionPath } from './shell/router.ts';
import { Feedback,describeAdminError } from './shell/Feedback.tsx';
import { DataTable } from './shell/DataTable.tsx';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';
export function SigninGeographyPage(props:{load?:()=>Promise<SigninGeography>}):JSX.Element{
 const shell=useContext(ShellContext);const [report,setReport]=createSignal<SigninGeography>();const [busy,setBusy]=createSignal(true);const [error,setError]=createSignal('');
 async function reload():Promise<void>{setBusy(true);try{setReport(await(props.load??loadSigninGeography)());setError('');}catch(cause){setError(describeAdminError(cause));}finally{setBusy(false);}}
 onMount(()=>void reload());
 const label=(value:string):string=>t(`admin-obs-geo-value-${value}`);
 const rows=():{id:string;value:string}[]=>{const r=report();return r?[
 {id:'source',value:r.sourceIp??t('admin-obs-geo-value-unknown')},{id:'country',value:r.country??t('admin-obs-geo-value-unknown')},
 {id:'sessions',value:t(r.enforceSessions?'admin-obs-geo-sessions-on':'admin-obs-geo-sessions-off')},{id:'database',value:label(r.database)},{id:'scope',value:label(r.scope)},{id:'mode',value:label(r.mode)},
 {id:'countries',value:r.countries.join(', ')||t('admin-obs-geo-value-empty')},{id:'unknown',value:label(r.unknown)},
 {id:'admin',value:label(r.administratorDecision)},{id:'mailbox',value:label(r.mailboxDecision)}]:[];};
 return <section class={css.section} aria-label={t('admin-obs-geo-title')}>
 <div class={ux.pageHead}><h2 class={css.heading}>{t('admin-obs-geo-title')}</h2><div class={ux.pageActions}><button class="btn btn--ghost" onClick={()=>void shell?.router.navigate(sectionPath('security'))}>{t('admin-security-center-title')}</button><button class="btn" disabled={busy()} onClick={()=>void reload()}>{t('admin-obs-geo-refresh')}</button></div></div>
 <p class={css.note}>{t(report()?.enforceSessions?'admin-obs-geo-session-note':'admin-obs-geo-note')}</p>
 <Show when={report()?.database==='unavailable'}><Feedback kind="info" message={t('admin-obs-geo-unavailable')}/></Show>
 <Show when={shell?.can.can('settings.read')}><div class={ux.pageActions}><button class="btn" onClick={()=>void shell?.router.navigate(sectionPath('settings',['security.signin.geoScope']))}>{t('admin-obs-geo-configure')}</button><button class="btn" onClick={()=>void shell?.router.navigate(sectionPath('settings',['security.signin.geoEnforceSessions']))}>{t('admin-obs-geo-configure-sessions')}</button></div></Show>
 <div class={css.card}><DataTable caption={t('admin-obs-geo-title')} rows={rows()} columns={[{id:'field',header:t('admin-obs-geo-field'),cell:row=>t(`admin-obs-geo-field-${row.id}`)},{id:'value',header:t('admin-obs-geo-value'),cell:row=>row.value}]} rowKey={row=>row.id} state={busy()?'loading':error()?'error':'ready'} errorText={error()} onRetry={()=>void reload()} emptyText={t('admin-obs-geo-empty')}/></div>
 </section>;
}
