import {For,Show,createSignal,onMount,onCleanup,type JSX} from 'solid-js';
import {nextcloudPimApi,type NextcloudPimApi,type CloudPimView,type CloudDiscovery,type CloudPimKind} from '../../api/nextcloudPim.ts';
import {AsyncPending} from '../../components/AsyncState.tsx';
import {t} from '../../i18n';
import * as css from './styles.css.ts';
export function NextcloudPim(props:{accountId:string;instanceId:string;instanceName:string;onBack:()=>void;api?:NextcloudPimApi}):JSX.Element {
  const api=()=>props.api??nextcloudPimApi;
  const [view,setView]=createSignal<CloudPimView>();const [items,setItems]=createSignal<CloudDiscovery[]>();
  const [kind,setKind]=createSignal<CloudPimKind>('calendar');const [busy,setBusy]=createSignal(false);const [error,setError]=createSignal('');const [notice,setNotice]=createSignal('');
  let alive=true;onCleanup(()=>{alive=false;});
  async function load():Promise<void>{const data=await api().list(props.accountId);if(alive&&data.accountId===props.accountId)setView(data);}
  async function run(action:()=>Promise<void>):Promise<void>{if(busy())return;setBusy(true);setError('');setNotice('');try{await action();}catch{if(alive)setError(t('nextcloud-pim-error'));}finally{if(alive)setBusy(false);}}
  onMount(()=>void run(load));
  async function discover():Promise<void>{const result=await api().discover(props.accountId,props.instanceId,kind());if(alive&&result.accountId===props.accountId)setItems(result.collections);}
  async function connect(item:CloudDiscovery):Promise<void>{await api().apply(props.accountId,props.instanceId,item);if(!alive)return;await load();if(alive)setNotice(t('nextcloud-pim-sync-needed'));}
  const configured=()=>view()?.collections.filter(row=>row.instanceId===props.instanceId)??[];
  return <section class={css.section}>
    <h2 class={css.heading}>{t('nextcloud-pim-title')} — {props.instanceName}</h2><p>{t('nextcloud-pim-help')}</p>
    <Show when={busy()&&!view()}><AsyncPending/></Show>
    <Show when={error()}><p role="alert">{error()}</p><button class={css.button} disabled={busy()} onClick={()=>void run(load)}>{t('nextcloud-account-reload')}</button></Show>
    <Show when={notice()}><p role="status">{notice()}</p></Show>
    <Show when={view()}>{data=><Show when={data().supported} fallback={<p>{t('nextcloud-pim-native-required')}</p>}>
      <label class={css.label}>{t('nextcloud-pim-category')}<select class={css.input} value={kind()} disabled={busy()} onChange={e=>{setKind(e.currentTarget.value as CloudPimKind);setItems(undefined);}}><option value="calendar">{t('nextcloud-pim-calendar')}</option><option value="contacts">{t('nextcloud-pim-contacts')}</option><option value="tasks">{t('nextcloud-pim-tasks')}</option></select></label>
      <div class={css.actions}><button class={css.button} disabled={busy()} onClick={()=>void run(discover)}>{t('nextcloud-pim-discover')}</button><button class={css.button} disabled={busy()||!configured().some(row=>row.enabled)} onClick={()=>void run(async()=>{await api().sync(props.accountId);if(alive)setNotice(t('nextcloud-pim-synced'));})}>{t('nextcloud-pim-sync')}</button></div>
      <Show when={items()}>{found=><section class={css.section}><h3>{t('nextcloud-pim-discovered')}</h3><Show when={found().length===0}><p>{t('nextcloud-pim-empty')}</p></Show><For each={found()}>{item=><div class={css.actions}><span>{item.name||t('nextcloud-pim-unnamed')}</span><button class={css.button} disabled={busy()} aria-label={`${t('nextcloud-pim-connect')} ${item.name}`} onClick={()=>void run(()=>connect(item))}>{t('nextcloud-pim-connect')}</button></div>}</For></section>}</Show>
      <section class={css.section}><h3>{t('nextcloud-pim-connected')}</h3><p>{t('nextcloud-pim-disconnect-help')}</p><For each={configured()}>{row=><div class={css.actions}><span>{row.name||t('nextcloud-pim-unnamed')} — {t(row.ready?'nextcloud-pim-ready':'nextcloud-pim-disconnected')}</span><button class={css.ghost} disabled={busy()||!row.enabled} onClick={()=>void run(async()=>{await api().disconnect(props.accountId,row.id);if(alive)await load();})}>{t('nextcloud-pim-disconnect')}</button></div>}</For></section>
    </Show>}</Show>
    <div class={css.actions}><button class={css.ghost} disabled={busy()} onClick={props.onBack}>{t('nextcloud-pim-back')}</button></div>
  </section>;
}
