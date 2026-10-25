import { AsyncPending } from '../../components/AsyncState.tsx';
import {For,Show,createMemo,createSignal,onMount,type JSX} from 'solid-js';
import {adminTotpApi,type AdminTotpApi,type AdminTotpSetup} from '../../api/admin/totp.ts';
import {loadCatalog,t} from '../../i18n';
import {encodeQr,qrToSvg} from '../../modules/keys/qr.ts';
import {ConfirmDialog} from './shell/ConfirmDialog.tsx';
import {describeAdminError,Feedback} from './shell/Feedback.tsx';
import {useDirtyGuard} from './shell/useDirtyGuard.ts';
import * as css from '../Settings/styles.css.ts';
import * as personal from './personal.css.ts';
export function AdminTotp(props:{api?:AdminTotpApi;back:()=>void}):JSX.Element {
 const api=props.api??adminTotpApi;const[loaded,setLoaded]=createSignal(false);const[enrolled,setEnrolled]=createSignal(false);const[remaining,setRemaining]=createSignal(0);const[setup,setSetup]=createSignal<AdminTotpSetup>();const[code,setCode]=createSignal('');const[codes,setCodes]=createSignal<string[]>([]);const[busy,setBusy]=createSignal(false);const[error,setError]=createSignal<string>();const[removing,setRemoving]=createSignal(false);const[rotating,setRotating]=createSignal(false);
 useDirtyGuard(()=>!!setup()||codes().length>0);
 const qr=createMemo(()=>{const value=setup();if(!value)return undefined;try{return`data:image/svg+xml,${encodeURIComponent(qrToSvg(encodeQr(value.otpauthUri)))}`;}catch{return undefined;}});
 async function refresh():Promise<void>{const status=await api.status();setEnrolled(status.totpEnrolled);setRemaining(status.recoveryRemaining);setLoaded(true);}
 async function run(action:()=>Promise<void>):Promise<void>{if(busy())return;setBusy(true);setError(undefined);try{await action();}catch(cause){setError(describeAdminError(cause));}finally{setBusy(false);}}
 onMount(()=>{void loadCatalog('admin-personal');void run(refresh);});
 async function confirm(event:SubmitEvent):Promise<void>{event.preventDefault();if(!/^\d{6}$/.test(code()))return;await run(async()=>{const result=await api.confirm(code());if(!result.ok)throw new Error('Authenticator confirmation was refused');setSetup(undefined);setCode('');setEnrolled(true);setCodes(result.recoveryCodes);await refresh();});}
 return <section class={css.section} aria-label={t('admin-personal-totp')}><div class={css.actions}><button class={css.ghost} disabled={busy()} onClick={props.back}>{t('admin-totp-back')}</button></div><h2 class={css.heading}>{t('admin-personal-totp')}</h2><p class={css.prose}>{t('admin-totp-description')}</p><Feedback kind="error" message={error()}/><Show when={!loaded()}><Show when={busy()}><AsyncPending onRetry={()=>window.location.reload()}/></Show><Show when={error()}><button class={css.button} onClick={()=>void run(refresh)}>{t('admin-personal-retry')}</button></Show></Show>
 <Show when={loaded()}><p role="status">{t(enrolled()?'admin-personal-totp-enabled':'admin-personal-totp-disabled')}</p><Show when={codes().length===0} fallback={<section class={css.section} aria-label={t('admin-totp-recovery-title')}><h3 class={css.heading}>{t('admin-totp-recovery-title')}</h3><p>{t('admin-totp-recovery-once')}</p><ul><For each={codes()}>{value=><li><code>{value}</code></li>}</For></ul><button class={css.button} onClick={()=>setCodes([])}>{t('admin-totp-recovery-ack')}</button></section>}>
 <Show when={!enrolled()} fallback={<><p>{t('admin-totp-recovery-count',{count:remaining()})}</p><div class={css.actions}><button class={css.button} disabled={busy()} onClick={()=>setRotating(true)}>{t('admin-totp-recovery-replace')}</button><button class={css.danger} disabled={busy()} onClick={()=>setRemoving(true)}>{t('admin-personal-totp-remove')}</button></div></>}>
 <Show when={setup()} keyed fallback={<button class={css.button} disabled={busy()} onClick={()=>void run(async()=>{setSetup(await api.begin());setCode('');})}>{t('admin-personal-totp-start')}</button>}>{value=><form class={css.stack} onSubmit={event=>void confirm(event)}><p>{t('admin-totp-setup-help')}</p><Show when={qr()}>{source=><img class={personal.totpQr} src={source()} alt={t('admin-totp-qr-alt')}/>}</Show><label class={css.field}>{t('admin-totp-manual-key')}<input class={css.input} readOnly value={value.secret} autocomplete="off"/></label><label class={css.field}>{t('admin-personal-totp-code')}<input class={css.input} required pattern="[0-9]{6}" maxlength={6} autocomplete="one-time-code" inputmode="numeric" value={code()} disabled={busy()} onInput={event=>setCode(event.currentTarget.value)}/></label><div class={css.actions}><button class={css.button} type="submit" disabled={busy()||!/^\d{6}$/.test(code())}>{t('admin-personal-totp-confirm')}</button></div></form>}</Show>
 </Show></Show></Show>
 <ConfirmDialog open={removing()} title={t('admin-personal-totp-remove')} body={t('admin-totp-remove-detail')} confirmLabel={t('admin-personal-totp-remove')} onCancel={()=>setRemoving(false)} onConfirm={async()=>{await api.remove();setRemoving(false);setEnrolled(false);await run(refresh);}}/>
 <ConfirmDialog open={rotating()} title={t('admin-totp-recovery-replace')} body={t('admin-totp-recovery-shared')} confirmLabel={t('admin-totp-recovery-replace')} onCancel={()=>setRotating(false)} onConfirm={async()=>{const result=await api.rotate();setRotating(false);setCodes(result.recoveryCodes);await run(refresh);}}/>
 </section>;
}
