import {createEffect,createSignal,For,onCleanup,onMount,Show,type JSX} from 'solid-js';
import {packagesApi,PackageError,type PackagesApi,type PackagesView,type PackageDraft,type TrackedPackage} from '../../api/packages.ts';
import {Icon} from '../../components/Icon.tsx';
import {t,loadCatalog} from '../../i18n';
import type {SettingsNavigation} from './navigation.tsx';
import * as css from './styles.css.ts';

function PackageEditor(props:{value?:TrackedPackage;disabled:boolean;save:(value:PackageDraft,id?:string)=>Promise<boolean>;navigation:SettingsNavigation}):JSX.Element {
  const initial={label:props.value?.label??'',trackingNumber:props.value?.trackingNumber??'',carrier:props.value?.carrier??''};
  const [draft,setDraft]=createSignal(initial);const [baseline,setBaseline]=createSignal(JSON.stringify(initial));
  onCleanup(props.navigation.beforeLeave(()=>JSON.stringify(draft())===baseline()||window.confirm(t('packages-discard'))));
  async function submit(event:SubmitEvent):Promise<void>{event.preventDefault();if(await props.save(draft(),props.value?.id)){setBaseline(JSON.stringify(draft()));await props.navigation.navigate({section:'packages'});}}
  return <form class={css.section} onSubmit={event=>void submit(event)}>
    <h3 class={css.heading}>{t(props.value?'packages-edit':'packages-add')}</h3>
    <For each={['label','trackingNumber','carrier'] as const}>{key=><label class={css.field}><span>{t(`packages-${key}`)}</span><input class={css.input} aria-label={t(`packages-${key}`)} maxlength={key==='label'?128:key==='carrier'?80:100} required={key!=='label'} value={draft()[key]} disabled={props.disabled} onInput={event=>setDraft(value=>({...value,[key]:event.currentTarget.value}))} aria-describedby={`package-${key}-help`}/><small id={`package-${key}-help`} class={css.meta}>{t(`packages-${key}-help`)}</small></label>}</For>
    <p class={css.meta}>{t('packages-save-local')}</p>
    <div class={css.actions}><button type="submit" class={css.button} disabled={props.disabled||JSON.stringify(draft())===baseline()}>{t('packages-save')}</button><button type="button" class={css.ghost} onClick={()=>void props.navigation.navigate({section:'packages'})}>{t('packages-back')}</button></div>
    <Show when={props.value}><h4>{t('packages-history')}</h4><div style={{'overflow-x':'auto'}}><table class="settings-shortcut-table" style={{'min-width':'38rem'}}><thead><tr><th>{t('packages-time')}</th><th>{t('packages-location')}</th><th>{t('packages-event')}</th></tr></thead><tbody><For each={props.value?.checkpoints??[]}>{row=><tr><td>{row.time}</td><td>{row.location}</td><td>{row.message}</td></tr>}</For></tbody></table></div></Show>
  </form>;
}
export function Packages(props:{accountId:string;navigation:SettingsNavigation;api?:PackagesApi}):JSX.Element {
  const [now,setNow]=createSignal(Date.now()/1000);onMount(()=>{const timer=window.setInterval(()=>setNow(Date.now()/1000),1000);onCleanup(()=>window.clearInterval(timer));});
  const [view,setView]=createSignal<PackagesView>();const [busy,setBusy]=createSignal(false);const [error,setError]=createSignal('');const [notice,setNotice]=createSignal('');
  let generation=0;let alive=true;onCleanup(()=>{generation++;alive=false;});
  const api=()=>props.api??packagesApi(props.accountId);
  async function reload():Promise<void>{const epoch=++generation;const account=props.accountId;setBusy(true);setError('');try{const result=await api().load();if(epoch===generation&&account===props.accountId)setView(result);}catch(e){if(epoch===generation)setError(message(e));}finally{if(epoch===generation)setBusy(false);}}
  function message(error:unknown):string {
    const allowed=['packageUnavailable','packageChanged','packageDisabled','invalidPackage','packageLimit','packageDuplicate','packageNotFound','packageBusy','packageCooldown','packageProviderUnavailable','packageProviderResponse'];
    const code=error instanceof PackageError&&allowed.includes(error.code)?error.code:'packageUnavailable';
    return t(`packages-error-${code}`);
  }
  async function run(operation:()=>Promise<void>,success:string):Promise<boolean>{if(busy())return false;const account=props.accountId;setBusy(true);setError('');setNotice('');try{await operation();if(!alive||account!==props.accountId)return false;await reload();if(!alive||account!==props.accountId)return false;setNotice(t(success));return true;}catch(e){if(alive&&account===props.accountId){setError(message(e));const current=await api().load().catch(()=>undefined);if(alive&&account===props.accountId&&current)setView(current);}return false;}finally{if(alive&&account===props.accountId)setBusy(false);}}
  createEffect(()=>{setView(undefined);void reload();});onMount(()=>void loadCatalog('packages'));
  const disabled=()=>busy()||!view()?.available||!view()?.state.enabled;
  const editor=()=>props.navigation.route().editor;
  const selected=()=>view()?.state.packages.find(row=>row.id===editor()?.id);
  async function action(row:TrackedPackage,kind:'refresh'|'remove'):Promise<void>{const current=view();if(!current?.revision)return;if(kind==='remove'&&!window.confirm(t('packages-remove-confirm',{name:row.label||row.trackingNumber})))return;await run(()=>api().action(row.id,current.revision!,kind),kind==='refresh'?'packages-refreshed':'packages-removed');}
  return <section class={css.section} aria-label={t('packages-title')}>
    <h2 class={css.heading}>{t('packages-title')}</h2>
    <Show when={error()}><p role="alert">{error()}</p></Show><Show when={notice()}><p role="status">{notice()}</p></Show>
    <Show when={view()} fallback={<p role="status">{t('packages-loading')}</p>}>{current=><>
      <Show when={editor()} keyed fallback={<>
        <p class={css.prose}>{t('packages-privacy')}</p><Show when={!current().available}><p role="status">{t('packages-unavailable')}</p></Show>
        <label class={css.check}><input class={css.checkbox} type="checkbox" checked={current().state.enabled} disabled={busy()||(!current().available&&!current().state.enabled)} onChange={event=>{const enabled=event.currentTarget.checked;void run(()=>api().preference(enabled,current().revision),'packages-preferences-saved');}}/>{t('packages-enable')}</label>
        <div class={css.actions}><button class={css.button} disabled={disabled()} onClick={()=>void props.navigation.navigate({section:'packages',editor:{kind:'new'}})}><Icon name="plus"/>{t('packages-add')}</button><button class={css.ghost} disabled={busy()} aria-label={t('packages-reload')} onClick={()=>void reload()}><Icon name="refresh"/></button></div>
        <div style={{'overflow-x':'auto'}}><table class="settings-shortcut-table" style={{'min-width':'38rem'}}><thead><tr><th>{t('packages-label')}</th><th>{t('packages-trackingNumber')}</th><th>{t('packages-status')}</th><th>{t('packages-estimate')}</th><th>{t('packages-actions')}</th></tr></thead><tbody><For each={current().state.packages}>{row=><tr><td>{row.label||row.carrier}</td><td>{row.trackingNumber}<br/><small>{row.carrier}</small></td><td>{t(`packages-status-${row.status}`)}</td><td>{row.estimatedDelivery??'—'}</td><td><div class={css.actions}><button class={css.ghost} disabled={busy()} aria-label={t('packages-edit-name',{name:row.label||row.trackingNumber})} onClick={()=>void props.navigation.navigate({section:'packages',editor:{kind:'edit',id:row.id}})}><Icon name="edit"/></button><button class={css.ghost} disabled={disabled()||(row.attemptedAt!==null&&now()-row.attemptedAt<current().refreshSecs)} aria-label={t('packages-refresh-name',{name:row.label||row.trackingNumber})} onClick={()=>void action(row,'refresh')}><Icon name="refresh"/></button><button class={css.ghost} disabled={busy()} aria-label={t('packages-remove-name',{name:row.label||row.trackingNumber})} onClick={()=>void action(row,'remove')}><Icon name="delete"/></button></div></td></tr>}</For></tbody></table></div>
        <p class={css.meta}>{t('packages-refresh-help',{seconds:current().refreshSecs})}</p>
      </>}>{route=><Show when={route.kind==='new'||selected()} fallback={<p role="alert">{t('packages-error-packageNotFound')}</p>}><PackageEditor {...(selected()?{value:selected()!}:{})} disabled={disabled()} navigation={props.navigation} save={(draft,id)=>run(()=>api().save(draft,current().revision,id),'packages-saved')}/></Show>}</Show>
    </>}</Show>
  </section>;
}
