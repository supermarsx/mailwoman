// Settings backups sent by this server to an administrator-owned Nextcloud folder (t64).
// The app password is write-only: the server answers `hasAppPassword`, never the value.
import {Show,createEffect,createSignal,useContext,type JSX} from 'solid-js';
import {adminRequest} from './shell/adminApi.ts';
import {ShellContext} from './shell/context.ts';
import {describeAdminError} from './shell/Feedback.tsx';
import {Icon} from '../../components/Icon.tsx';
import {formatNumber,t} from '../../i18n';
import * as css from './admin.css.ts';

export interface NextcloudBackupConfig {baseUrl:string;username:string;folder:string;keep:number;keepDays:number|null;caPem:string;hasAppPassword:boolean;webdavUrl:string|null}
export interface NextcloudBackupFailure {ok:false;step:string;code:string;status:number|null}
export interface NextcloudBackupView {config:NextcloudBackupConfig|null;lastRun:({at:number;ok:boolean;file?:string;bytes?:number;removed?:number|null;step?:string;code?:string;status?:number|null})|null;appPasswordHelp:string}
type TestResult={ok:true;folderCreated:boolean}|NextcloudBackupFailure;
type RunResult={ok:true;file:string;bytes:number;removed:string[];folder:string}|NextcloudBackupFailure;
export const APP_PASSWORD_DOCS='https://docs.nextcloud.com/server/latest/user_manual/en/session_management.html#managing-devices';
const PATH='/config/backup/nextcloud';
const empty={baseUrl:'',username:'',folder:'Backups/Mailwoman',keep:7,keepDays:null as number|null,caPem:''};
const KNOWN=new Set(['authRejected','userNotFound','folderIsFile','folderCreateDenied','writeDenied','fileExists','quotaExceeded','verifyMismatch','deleteDenied','redirected','refused','egressBlocked','timeout','tooLarge','invalidRequest','unreachable','noAppPassword','credentialUnreadable','invalidCa','invalidListing']);

/** One sentence for a failed step: what failed, and what to check. */
export function describeFailure(failure:{step?:string;code?:string;status?:number|null}):string{
  const code=failure.code&&KNOWN.has(failure.code)?failure.code:'refused';
  return t('admin-ncbackup-step-failed',{step:t(`admin-ncbackup-step-${failure.step??'config'}`),reason:t(`admin-ncbackup-error-${code}`,{status:failure.status??0})});
}

export function ConfigBackupNextcloud():JSX.Element{
  const shell=useContext(ShellContext);
  const allowed=()=>shell?.can.can('config.export')??false;
  const [view,setView]=createSignal<NextcloudBackupView>();
  const [etag,setEtag]=createSignal<string|null>(null);
  const [form,setForm]=createSignal({...empty});
  const [password,setPassword]=createSignal('');
  const [busy,setBusy]=createSignal(false);
  const [error,setError]=createSignal('');
  const [status,setStatus]=createSignal('');
  function accept(data:NextcloudBackupView,tag:string|null){setView(data);setEtag(tag);setPassword('');const config=data.config;setForm(config?{baseUrl:config.baseUrl,username:config.username,folder:config.folder,keep:config.keep,keepDays:config.keepDays,caPem:config.caPem}:{...empty});}
  async function reload(){try{const result=await adminRequest<NextcloudBackupView>(PATH);accept(result.data,result.etag);}catch(cause){setError(describeAdminError(cause));}}
  // The grant arrives with the shell's session; read once it is known.
  const ready=()=>{const can=shell?.can as {loaded?:()=>boolean}|undefined;return can?.loaded?.()??true;};
  let loaded=false;createEffect(()=>{if(ready()&&allowed()&&!loaded){loaded=true;void reload();}});
  const change=(patch:Partial<typeof empty>)=>{setStatus('');setForm(value=>({...value,...patch}));};
  async function act(action:()=>Promise<void>){setBusy(true);setError('');setStatus('');try{await action();}catch(cause){setError(describeAdminError(cause));}finally{setBusy(false);}}
  const save=(event:SubmitEvent)=>{event.preventDefault();void act(async()=>{const value=form();const result=await adminRequest<NextcloudBackupView>(PATH,{method:'PUT',ifMatch:etag(),body:{...value,keepDays:value.keepDays||null,...(password()?{appPassword:password()}:{})}});accept(result.data,result.etag);setStatus(t('admin-ncbackup-saved'));});};
  const test=()=>void act(async()=>{const result=await adminRequest<TestResult>(`${PATH}/test`,{method:'POST'});if(result.data.ok)setStatus(t(result.data.folderCreated?'admin-ncbackup-test-ok-created':'admin-ncbackup-test-ok'));else setError(describeFailure(result.data));});
  const run=()=>void act(async()=>{const result=await adminRequest<RunResult>(`${PATH}/run`,{method:'POST'});const data=result.data;if(data.ok)setStatus(t('admin-ncbackup-run-ok',{file:data.file,folder:data.folder,bytes:formatNumber(data.bytes),removed:data.removed.length}));else setError(describeFailure(data));await reload();});
  const remove=()=>void act(async()=>{const result=await adminRequest<NextcloudBackupView>(PATH,{method:'DELETE',ifMatch:etag()});accept(result.data,result.etag);setStatus(t('admin-ncbackup-removed'));});
  const configured=()=>!!view()?.config;
  const dirty=()=>{const config=view()?.config;const value=form();return password()!==''||!config||config.baseUrl!==value.baseUrl||config.username!==value.username||config.folder!==value.folder||config.keep!==value.keep||(config.keepDays??null)!==(value.keepDays||null)||config.caPem!==value.caPem;};
  return <section class={css.card} aria-labelledby="ncbackup-title">
    <h3 id="ncbackup-title"><Icon name="nextcloud"/> {t('admin-ncbackup-title')}</h3>
    <p>{t('admin-ncbackup-help')}</p>
    <Show when={allowed()} fallback={<p class={css.note}>{t('admin-ncbackup-no-permission')}</p>}>
    <Show when={error()}><p role="alert">{error()}</p></Show>
    <Show when={status()}><p role="status">{status()}</p></Show>
    <form onSubmit={save}>
      <label class="field"><span>{t('admin-ncbackup-url')}</span><input type="url" required maxlength={2048} placeholder="https://cloud.example.com" value={form().baseUrl} disabled={busy()} onInput={event=>change({baseUrl:event.currentTarget.value})}/><small>{t('admin-ncbackup-url-help')}</small></label>
      <label class="field"><span>{t('admin-ncbackup-username')}</span><input required maxlength={256} autocomplete="off" value={form().username} disabled={busy()} onInput={event=>change({username:event.currentTarget.value})}/></label>
      <label class="field"><span>{t('admin-ncbackup-password')}</span><input type="password" autocomplete="new-password" maxlength={256} required={!view()?.config?.hasAppPassword} value={password()} disabled={busy()} onInput={event=>{setStatus('');setPassword(event.currentTarget.value);}}/><small>{t(view()?.config?.hasAppPassword?'admin-ncbackup-password-stored':'admin-ncbackup-password-help')} <a href={view()?.appPasswordHelp??APP_PASSWORD_DOCS} target="_blank" rel="noopener noreferrer">{t('admin-ncbackup-password-docs')}</a></small></label>
      <label class="field"><span>{t('admin-ncbackup-folder')}</span><input required maxlength={512} value={form().folder} disabled={busy()} onInput={event=>change({folder:event.currentTarget.value})}/><small>{t('admin-ncbackup-folder-help')}</small></label>
      <label class="field"><span>{t('admin-ncbackup-keep')}</span><input type="number" min="1" max="1000" required value={form().keep} disabled={busy()} onInput={event=>change({keep:Number(event.currentTarget.value)})}/></label>
      <label class="field"><span>{t('admin-ncbackup-keep-days')}</span><input type="number" min="1" max="36500" value={form().keepDays??''} disabled={busy()} onInput={event=>change({keepDays:event.currentTarget.value?Number(event.currentTarget.value):null})}/><small>{t('admin-ncbackup-keep-help')}</small></label>
      <details><summary>{t('admin-ncbackup-ca')}</summary><label class="field"><span>{t('admin-ncbackup-ca-label')}</span><textarea rows={4} maxlength={65536} value={form().caPem} disabled={busy()} onInput={event=>change({caPem:event.currentTarget.value})}/><small>{t('admin-ncbackup-ca-help')}</small></label></details>
      <Show when={view()?.config?.webdavUrl}>{url=><p class={css.note}>{t('admin-ncbackup-webdav')} <code>{url()}</code></p>}</Show>
      <div><button class="btn btn--primary" type="submit" disabled={busy()||!dirty()}>{t('admin-ncbackup-save')}</button> <button class="btn" type="button" disabled={busy()||!configured()||dirty()} onClick={test}>{t('admin-ncbackup-test')}</button> <button class="btn" type="button" disabled={busy()||!configured()||dirty()} onClick={run}><Icon name="nextcloud"/>{t('admin-ncbackup-run')}</button> <Show when={configured()}><button class="btn" type="button" disabled={busy()} onClick={remove}>{t('admin-ncbackup-remove')}</button></Show></div>
    </form>
    <Show when={view()?.lastRun}>{last=><p class={css.note}>{last().ok?t('admin-ncbackup-last-ok',{file:last().file??'',at:new Date(last().at*1000).toLocaleString()}):t('admin-ncbackup-last-failed',{at:new Date(last().at*1000).toLocaleString(),reason:describeFailure(last())})}</p>}</Show>
    </Show>
  </section>;
}
