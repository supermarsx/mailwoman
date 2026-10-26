import {For,Show,createSignal,onMount,onCleanup,type JSX} from 'solid-js';
import {NextcloudService} from './service.ts';
import {t} from '../../i18n';
import {AsyncPending} from '../../components/AsyncState.tsx';
import * as css from './styles.css.ts';
/** Picker-local selection: no global account or cloud selection is mutated. */
export function InstanceSelector(props:{service:NextcloudService;onChange:(id:string)=>void;disabled?:boolean|undefined}):JSX.Element {
  const [instances,setInstances]=createSignal<{id:string;name:string}[]>();
  const [selected,setSelected]=createSignal('');const [error,setError]=createSignal(false);
  let alive=true;onCleanup(()=>{alive=false;});
  function select(id:string):void {if(!id)return;setSelected(id);props.service.selectInstance(id);props.onChange(id);}
  onMount(()=>{void props.service.instances().then(rows=>{if(!alive)return;setInstances(rows);if(rows.length===1)select(rows[0]!.id);}).catch(()=>{if(alive)setError(true);});});
  return <Show when={!error()} fallback={<p role="alert" class={css.error}>{t('nextcloud-instances-error')}</p>}>
    <Show when={instances()} fallback={<AsyncPending/>}>{rows=><Show when={rows().length>0} fallback={<p>{t('nextcloud-instances-empty')}</p>}>
      <label>{t('nextcloud-instance-select')}<select disabled={props.disabled} value={selected()} onChange={event=>select(event.currentTarget.value)}><option value="" disabled>{t('nextcloud-instance-placeholder')}</option><For each={rows()}>{row=><option value={row.id}>{row.name}</option>}</For></select></label>
    </Show>}</Show>
  </Show>;
}
