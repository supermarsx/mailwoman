// Admin › Security › Console address › Send by email (t58).
//
// The server sends one message with the console address (and, while a change is
// pending, the address that applies after the next restart) to one recipient:
// the admin's sign-in name when it is an address, a configured operator
// recipient, or an address typed here. The server builds the address, asks for
// step-up, audits and rate-limits the send; this dialog only picks the recipient.
import {For,Show,createResource,createSignal,type JSX} from 'solid-js';
import {consolePathMailApi,validRecipient,type ConsolePathMailApi} from '../../api/admin/consolePath.ts';
import {activeLocaleSignal} from '../../i18n/registry.ts';
import {loadCatalog,t} from '../../i18n';
import {toast} from '../../components/toast/toast.ts';
import {ConfirmDialog} from './shell/ConfirmDialog.tsx';
import {AdminProblemError} from './shell/adminApi.ts';
import {describeAdminError} from './shell/Feedback.tsx';
import * as css from './admin.css.ts';
import * as ux from './shell/ux.css.ts';

class InvalidRecipient extends Error {}

const OTHER='other';

export function describeMailError(cause:unknown):string{
  if(cause instanceof InvalidRecipient)return t('admin-console-url-mail-invalid');
  if(cause instanceof AdminProblemError){
    if(cause.code==='invalidRecipient')return t('admin-console-url-mail-invalid');
    if(cause.code==='mailNotConfigured')return t('admin-console-url-mail-unavailable');
    if(cause.code==='rateLimited')return t('admin-console-url-mail-rate');
    if(cause.code==='publicAddressUnknown')return t('admin-console-url-mail-no-address');
  }
  return describeAdminError(cause);
}

export function ConsolePathMail(props:{pending:boolean;api?:ConsolePathMailApi}):JSX.Element {
  void loadCatalog('admin-console-url-mail');
  const api=props.api??consolePathMailApi;
  // A failed status read leaves the button off; it never throws into the page.
  const [status]=createResource(async()=>{try{return (await api.status()).data;}catch{return null;}});
  const [open,setOpen]=createSignal(false);
  const [choice,setChoice]=createSignal('');
  const [other,setOther]=createSignal('');
  const configured=():boolean=>status()?.configured===true;
  const suggestions=()=>status()?.suggestions??[];
  const recipient=():string=>choice()===OTHER?other().trim():choice();
  function start():void{
    setChoice(suggestions()[0]?.address??OTHER);
    setOther('');
    setOpen(true);
  }
  async function send():Promise<void>{
    const to=recipient();
    if(!validRecipient(to))throw new InvalidRecipient();
    await api.send(to,activeLocaleSignal());
    setOpen(false);
    toast.success(t('admin-console-url-mail-sent',{address:to}));
  }
  const hintId='console-path-mail-hint';
  return <>
    <button type="button" class="btn" data-testid="console-path-mail" disabled={!configured()} aria-describedby={status()&&!configured()?hintId:undefined} onClick={start}>{t('admin-console-url-mail-button')}</button>
    <Show when={status()&&!configured()}><p id={hintId} class={css.note}>{t('admin-console-url-mail-unavailable')}</p></Show>
    <ConfirmDialog open={open()} danger={false} testId="console-path-mail-dialog" title={t('admin-console-url-mail-title')} body={t(props.pending?'admin-console-url-mail-body-pending':'admin-console-url-mail-body')} confirmLabel={t('admin-console-url-mail-send')} describeError={describeMailError} onCancel={()=>setOpen(false)} onConfirm={send}>
      <fieldset class={ux.fieldset}>
        <legend class={ux.legend}>{t('admin-console-url-mail-recipient')}</legend>
        <For each={suggestions()}>{s=><label class={ux.checkboxRow}><input type="radio" name="console-path-mail-recipient" value={s.address} checked={choice()===s.address} onChange={()=>setChoice(s.address)}/> <span>{t(s.kind==='signIn'?'admin-console-url-mail-signin':'admin-console-url-mail-operator',{address:s.address})}</span></label>}</For>
        <Show when={suggestions().length>0}><label class={ux.checkboxRow}><input type="radio" name="console-path-mail-recipient" value={OTHER} checked={choice()===OTHER} onChange={()=>setChoice(OTHER)}/> <span>{t('admin-console-url-mail-other')}</span></label></Show>
        <Show when={choice()===OTHER}>
          <label class="field"><span>{t('admin-console-url-mail-other-label')}</span>
            <input class={ux.control} aria-label={t('admin-console-url-mail-other-label')} aria-describedby="console-path-mail-other-help" type="text" inputmode="email" autocomplete="email" spellcheck={false} value={other()} aria-invalid={other()!==''&&!validRecipient(other())} onInput={event=>setOther(event.currentTarget.value)}/>
            <small id="console-path-mail-other-help">{t('admin-console-url-mail-other-help')}</small>
          </label>
        </Show>
      </fieldset>
      <p class={css.note}>{t('admin-console-url-mail-limit',{count:status()?.perAdminPerHour??3})}</p>
    </ConfirmDialog>
  </>;
}
