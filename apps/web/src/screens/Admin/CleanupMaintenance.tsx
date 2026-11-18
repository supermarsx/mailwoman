import {createSignal,type JSX} from 'solid-js';
import {loadCatalog,t} from '../../i18n';
import {adminRequest} from './shell/adminApi.ts';
import {Feedback,describeAdminError} from './shell/Feedback.tsx';
import {Icon} from '../../components/Icon.tsx';
import * as css from './admin.css.ts';

/** Live log maintenance follows configured limits; database repair runs offline. */
export function CleanupMaintenance():JSX.Element{
 void loadCatalog('admin-database');
 const [busy,setBusy]=createSignal(false),[failure,setFailure]=createSignal<string|null>(null),[success,setSuccess]=createSignal<string|null>(null);
 async function logs(rotate:boolean):Promise<void>{
  if(busy())return;setBusy(true);setFailure(null);setSuccess(null);
  try{await adminRequest('/maintenance/logs',{method:'POST',body:{rotate}});setSuccess(t('admin-db-maintenance-requested'));}
  catch(error){setFailure(describeAdminError(error));}finally{setBusy(false);}
 }
 async function copy(command:string):Promise<void>{
  setFailure(null);setSuccess(null);
  try{await navigator.clipboard.writeText(command);setSuccess(t('admin-db-command-copied'));}catch(error){setFailure(describeAdminError(error));}
 }
 const command=(label:string,value:string)=><div class={css.card}><h4 class={css.heading}>{label}</h4><p><code>{value}</code></p><button class="btn btn--ghost" type="button" onClick={()=>void copy(value)}><Icon name="copy"/>{t('admin-db-copy-command')}</button></div>;
 return <section class={css.section} aria-label={t('admin-db-cleanup')}>
  <h2 class={css.heading}>{t('admin-db-cleanup')}</h2>
  <Feedback kind="error" as="toast" message={failure()}/><Feedback kind="success" as="toast" message={success()}/>
  <section class={css.card}><h3 class={css.heading}>{t('admin-db-log-files')}</h3><p class={css.note}>{t('admin-db-log-help')}</p>
   <div class={css.headingRow}><button class="btn btn--ghost" type="button" disabled={busy()} onClick={()=>void logs(false)}><Icon name="delete"/>{t('admin-db-cleanup')}</button><button class="btn" type="button" disabled={busy()} onClick={()=>void logs(true)}><Icon name="refresh"/>{t('admin-db-rotate')}</button></div>
  </section>
  <section class={css.card}><h3 class={css.heading}>{t('admin-db-offline-cleanup')}</h3><p class={css.note}>{t('admin-db-offline-help')}</p>
   <div class={css.grid}>{command(t('admin-db-preview-command'),'./deploy.sh maintenance --orphan-bodies --compact')}{command(t('admin-db-apply-command'),'./deploy.sh maintenance --apply --orphan-bodies --compact')}</div>
   <p class={css.note}><code>--purge-search-index</code> · <code>--logs-older-than-days N</code> · <code>--skip-backup</code></p>
  </section>
 </section>;
}
