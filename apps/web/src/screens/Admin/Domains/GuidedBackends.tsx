import { EwsFields } from './EwsFields.tsx';
import {For,Show,createSignal,type JSX} from 'solid-js';
import {domainsApi,type DomainBackend,type DomainEndpoint,type DomainStatus,type DomainsApi,type UsernameFormat} from '../../../api/admin/domains.ts';
import {t} from '../../../i18n';
import {Icon} from '../../../components/Icon.tsx';
import {TransportFields} from './TransportFields.tsx';
import {AuthFields} from './AuthFields.tsx';
import {newGuidedBackend} from './guidedBackendModel.ts';
import * as css from '../../Settings/styles.css.ts';
import * as ux from '../shell/ux.css.ts';
import * as layout from './guidedDomain.css.ts';

/** Every edit remains in the parent wizard draft until its final create request. */
export function GuidedBackends(props:{value:readonly DomainBackend[];change:(rows:DomainBackend[])=>void;status:DomainStatus;domain:string;api:DomainsApi}):JSX.Element {
  const [selected,setSelected]=createSignal(-1),[preview,setPreview]=createSignal(''),[error,setError]=createSignal(''),[busy,setBusy]=createSignal(false);
  const protocols=()=>props.status.mode==='proxy'?['jmap'] as const:['imap','pop3','smtp','sieve','ews'] as const;
  const row=()=>props.value[selected()];
  function patch(value:Partial<DomainBackend>):void {
    const index=selected(),current=row();if(!current)return;
    const next={...current,...value};
    if(value.protocol){delete next.endpoint;delete next.sessionUrl;delete next.ews;if(value.endpoint)next.endpoint=value.endpoint;if(value.sessionUrl!==undefined)next.sessionUrl=value.sessionUrl;if(value.ews)next.ews=value.ews;}
    props.change(props.value.map((entry,i)=>i===index?next:next.preferred&&entry.role===next.role?{...entry,preferred:false}:entry));
    setPreview('');setError('');
  }
  const wire=():DomainEndpoint=>row()?.endpoint??newGuidedBackend('imap','').endpoint!;
  const username=():UsernameFormat=>row()?.username??{format:'fullEmail'};
  const endpoint=(value:Partial<DomainEndpoint>)=>patch({endpoint:{...wire(),...value}});
  function add():void {const index=props.value.length;let n=1;while(props.value.some(r=>r.id===`backend-${n}`))n++;props.change([...props.value,newGuidedBackend(protocols()[0],`backend-${n}`)]);setSelected(index);}
  function protocol(value:DomainBackend['protocol']):void {const current=row();if(current)patch({...newGuidedBackend(value,current.id),priority:current.priority??100,enabled:current.enabled!==false,preferred:current.preferred??false});}
  function format(value:UsernameFormat['format']):void {patch({username:value==='domainUser'?{format:value,domain:''}:value==='userRealm'?{format:value,realm:''}:value==='custom'?{format:value,template:'{email}'}:{format:value}});}
  async function previewUsername():Promise<void>{const source=JSON.stringify([props.domain,row()?.id,username()]);setBusy(true);setError('');try{const result=await (props.api.usernamePreview??domainsApi.usernamePreview)(`user@${props.domain}`,username());if(source===JSON.stringify([props.domain,row()?.id,username()]))setPreview(result.authenticationUsername);}catch(e){setError(e instanceof Error?e.message:String(e));}finally{setBusy(false);}}
  return <div class={css.stack}>
    <p class={css.meta}>{t('admin-domain-guided-draft-help')}</p>
    <Show when={row()} fallback={<>
      <div class={css.actions}><button type="button" class={css.button} disabled={props.value.length>=32} onClick={add}>{t('backend-editor-add')}</button></div>
      <Show when={props.value.length} fallback={<p>{t('admin-domain-guided-backend-empty')}</p>}>
        <div class={layout.tableWrap}><table class={layout.table} aria-label={t('admin-domain-guided-backend-list')}><thead><tr><th>{t('backend-editor-id')}</th><th>{t('backend-editor-protocol')}</th><th>{t('backend-editor-priority')}</th><th>{t('backend-editor-preferred')}</th><th>{t('admin-domain-guided-actions')}</th></tr></thead><tbody><For each={props.value}>{(entry,index)=><tr><td>{entry.id}<Show when={entry.enabled===false}><small> · {t('admin-domain-guided-disabled')}</small></Show></td><td>{entry.protocol.toUpperCase()}</td><td>{entry.priority??100}</td><td>{entry.preferred?'✓':'—'}</td><td><div class={layout.rowActions}><button type="button" class={ux.iconAction} aria-label={t('admin-domain-guided-edit-backend',{id:entry.id})} data-tooltip={t('backend-editor-edit')} onClick={()=>{setSelected(index());setPreview('');setError('');}}><Icon name="edit"/></button><button type="button" class={ux.iconAction} aria-label={t('admin-domain-guided-remove-backend',{id:entry.id})} data-tooltip={t('admin-domain-guided-remove-help')} onClick={()=>props.change(props.value.filter((_,i)=>i!==index()))}><Icon name="delete"/></button></div></td></tr>}</For></tbody></table></div>
      </Show>
    </>}>{current=><section class={css.stack} aria-label={t('backend-editor-edit')}>
      <div class={css.actions}><button type="button" class={css.button} onClick={()=>setSelected(-1)}>{t('admin-domain-guided-backend-done')}</button></div>
      <fieldset class={css.section}><legend class={css.heading}>{t('backend-editor-identity')}</legend><div class={layout.grid}>
        <label class={css.field}>{t('backend-editor-id')}<input class={css.input} maxlength="64" value={current().id} onInput={e=>patch({id:e.currentTarget.value})}/><small>{t('backend-editor-id-help')}</small></label>
        <label class={css.field}>{t('backend-editor-protocol')}<select class={css.select} value={current().protocol} onChange={e=>protocol(e.currentTarget.value as DomainBackend['protocol'])}><For each={protocols()}>{p=><option value={p}>{p.toUpperCase()}</option>}</For></select><small>{t('backend-editor-protocol-help')}</small></label>
        <label class={css.field}>{t('backend-editor-priority')}<input class={css.input} type="number" min="0" max="65535" value={current().priority??100} onInput={e=>patch({priority:e.currentTarget.valueAsNumber})}/><small>{t('backend-editor-priority-help')}</small></label>
      </div><label class={layout.check}><input type="checkbox" checked={current().enabled!==false} onChange={e=>patch({enabled:e.currentTarget.checked,...(!e.currentTarget.checked?{preferred:false}:{})})}/>{t('backend-editor-enabled')}</label><label class={layout.check}><input type="checkbox" checked={current().preferred??false} disabled={current().enabled===false} onChange={e=>patch({preferred:e.currentTarget.checked})}/>{t('backend-editor-preferred')}</label><p class={css.meta}>{t('backend-editor-preferred-help')}</p></fieldset>
      <fieldset class={css.section}><legend class={css.heading}>{t('backend-editor-connection')}</legend>
        <Show when={current().protocol==='ews'} fallback={<Show when={current().protocol==='jmap'} fallback={<><div class={layout.grid}>
          <label class={css.field}>{t('backend-editor-host')}<input class={css.input} value={wire().host} onInput={e=>endpoint({host:e.currentTarget.value})}/></label>
          <label class={css.field}>{t('backend-editor-port')}<input class={css.input} type="number" min="1" max="65535" value={wire().port} onInput={e=>endpoint({port:e.currentTarget.valueAsNumber})}/></label>
          <label class={css.field}>{t('backend-editor-security')}<select class={css.select} value={wire().tls} onChange={e=>endpoint({tls:e.currentTarget.value as DomainEndpoint['tls']})}><option value="implicit">TLS</option><option value="starttls">STARTTLS</option><option value="none">{t('backend-editor-plaintext')}</option></select></label>
        </div><Show when={wire().tls==='none'}><p role="alert">{t('backend-editor-plaintext-warning')}</p></Show><label class={layout.check}><input type="checkbox" checked={wire().allowPrivate??false} onChange={e=>endpoint({allowPrivate:e.currentTarget.checked})}/>{t('backend-editor-private')}</label><p class={css.meta}>{t('backend-editor-private-help')}</p>
        <TransportFields value={wire()} ready={props.status.transportSecurity?.runtimeReady===true} update={value=>patch({endpoint:value})}/>
        </>}><label class={css.field}>{t('backend-editor-session-url')}<input class={css.input} type="url" value={current().sessionUrl??''} onInput={e=>patch({sessionUrl:e.currentTarget.value})}/><small>{t('backend-editor-session-help')}</small></label></Show>}><EwsFields value={current().ews!} change={ews=>patch({ews})}/></Show>
      </fieldset>
      <Show when={current().protocol!=='jmap'}><fieldset class={css.section}><legend class={css.heading}>{t('backend-editor-auth')}</legend><p class={css.meta}>{t('backend-editor-auth-help')}</p>
        <Show when={current().protocol!=='ews'}><AuthFields value={wire().authentication} supported={props.status.authentication?.runtimeReady?props.status.authentication.protocols[current().protocol]:undefined} change={value=>{const next={...wire()};if(value)next.authentication=value;else delete next.authentication;patch({endpoint:next});}}/></Show>
        <label class={css.field}>{t('backend-editor-username-format')}<select class={css.select} value={username().format} onChange={e=>format(e.currentTarget.value as UsernameFormat['format'])}><For each={['fullEmail','localUser','domainUser','userRealm','custom'] as const}>{f=><option value={f}>{t(`backend-editor-format-${f}`)}</option>}</For></select><small>{t('backend-editor-username-help')}</small></label>
        <Show when={username().format==='domainUser'}><label class={css.field}>{t('backend-editor-auth-domain')}<input class={css.input} value={(username() as {domain:string}).domain} onInput={e=>patch({username:{format:'domainUser',domain:e.currentTarget.value}})}/></label></Show>
        <Show when={username().format==='userRealm'||username().format==='custom'}><label class={css.field}>{t('backend-editor-realm')}<input class={css.input} value={(username() as {realm?:string}).realm??''} onInput={e=>patch({username:{...username(),realm:e.currentTarget.value} as UsernameFormat})}/></label></Show>
        <Show when={username().format==='custom'}><label class={css.field}>{t('backend-editor-template')}<input class={css.input} maxlength="512" value={(username() as {template:string}).template} onInput={e=>patch({username:{...username(),template:e.currentTarget.value} as UsernameFormat})}/><small>{t('backend-editor-template-help')}</small></label></Show>
        <div class={css.actions}><button type="button" class={css.button} disabled={busy()} onClick={()=>void previewUsername()}>{t('backend-editor-preview-username')}</button></div><Show when={preview()}><p role="status">{t('backend-editor-preview-result')}: <code>{preview()}</code></p></Show><Show when={error()}><p role="alert">{error()}</p></Show>
      </fieldset></Show>
    </section>}</Show>
  </div>;
}
