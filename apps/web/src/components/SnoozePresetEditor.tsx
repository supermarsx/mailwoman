import {createSignal,For,Show,type JSX} from 'solid-js';
import {formatDateTime,loadCatalog,t} from '../i18n';
import {DEFAULT_SNOOZE_PRESETS,SNOOZE_KINDS,resolveSnooze,snoozePresetLabel,snoozeZone,scheduleWeekday,type SnoozeKind,type SnoozePreset} from '../state/snooze.ts';
import {Icon} from './Icon.tsx';
import * as table from '../screens/Settings/managementTable.css.ts';
import * as css from '../screens/Settings/styles.css.ts';
import * as ux from '../screens/Admin/shell/ux.css.ts';
void loadCatalog('snooze');
export function SnoozePresetEditor(props:{value:readonly SnoozePreset[];onChange:(rows:SnoozePreset[])=>void;disabled?:boolean|undefined;id?:string;describedBy?:string|undefined}):JSX.Element{
 const [selected,setSelected]=createSignal<string>();
 const current=()=>props.value.find(preset=>preset.id===selected());
 const patch=(value:Partial<SnoozePreset>):void=>props.onChange(props.value.map(preset=>preset.id===selected()?{...preset,...value}:preset));
 function kind(value:SnoozeKind):void{
  const preset=current();if(!preset)return;
  const {amount:_amount,time:_time,weekday:_weekday,ordinal:_ordinal,day:_day,...base}=preset;
  props.onChange(props.value.map(p=>p.id!==preset.id?p:{...base,kind:value,...(value==='weekday'||value==='nth-weekday'?{weekday:preset.weekday??1}:{}),...(value==='nth-weekday'?{ordinal:preset.ordinal??1}:{}),...(value==='day-of-month'?{day:preset.day??1}:{}),...(value==='hours'||value==='days'?{amount:preset.amount??2}:{}),...(value!=='hours'?{time:preset.time??'09:00'}:{})}));
 }
 function add():void{
  const id=crypto.randomUUID();props.onChange([...props.value,{id,kind:'hours',amount:2,enabled:true}]);setSelected(id);
 }
 function move(index:number,delta:number):void{const rows=[...props.value],other=index+delta;if(other<0||other>=rows.length)return;[rows[index],rows[other]]=[rows[other]!,rows[index]!];props.onChange(rows);}
 return <div id={props.id} aria-describedby={props.describedBy} class={css.stack}>
  <p class={css.meta}>{t('snooze-zone',{zone:snoozeZone()})}</p>
  <div class={table.bar}><span class={css.meta}>{props.value.length} / 32</span><button type="button" class={css.button} disabled={props.disabled||props.value.length>=32} onClick={add}><Icon name="plus"/>{t('snooze-add')}</button><button type="button" class={css.ghost} disabled={props.disabled} onClick={()=>{props.onChange(DEFAULT_SNOOZE_PRESETS.map(row=>({...row})));setSelected(undefined);}}>{t('snooze-restore')}</button></div>
  <div class={table.scroll}><table class={table.table}><thead><tr><th class={table.heading}>{t('snooze-preset')}</th><th class={`${table.heading} ${table.previewColumn}`}>{t('snooze-next-time')}</th><th class={`${table.heading} ${table.actionColumn}`}>{t('common-actions')}</th></tr></thead><tbody><For each={props.value}>{(preset,index)=><tr class={table.row} data-current={selected()===preset.id}>
   <td class={table.cell}><div class={table.name}><input type="checkbox" checked={preset.enabled} disabled={props.disabled} aria-label={`${t('snooze-enabled')}: ${snoozePresetLabel(preset)}`} onChange={event=>props.onChange(props.value.map(p=>p.id===preset.id?{...p,enabled:event.currentTarget.checked}:p))}/><button type="button" class={table.title} disabled={props.disabled} onClick={()=>setSelected(preset.id)}>{snoozePresetLabel(preset)}</button></div></td>
   <td class={`${table.cell} ${table.previewColumn}`}><span class={table.text}>{resolveSnooze(preset)?formatDateTime(resolveSnooze(preset)!):'—'}</span></td>
   <td class={table.cell}><div class={table.actions}><button type="button" class={`btn btn--ghost ${ux.iconAction}`} disabled={props.disabled||index()===0} title={t('snooze-up')} aria-label={`${t('snooze-up')}: ${snoozePresetLabel(preset)}`} onClick={()=>move(index(),-1)}><Icon name="chevronUp"/></button><button type="button" class={`btn btn--ghost ${ux.iconAction}`} disabled={props.disabled||index()===props.value.length-1} title={t('snooze-down')} aria-label={`${t('snooze-down')}: ${snoozePresetLabel(preset)}`} onClick={()=>move(index(),1)}><Icon name="chevronDown"/></button><button type="button" class={`btn btn--danger ${ux.iconAction}`} disabled={props.disabled} title={t('common-remove')} aria-label={`${t('common-remove')}: ${snoozePresetLabel(preset)}`} onClick={()=>{props.onChange(props.value.filter(p=>p.id!==preset.id));if(selected()===preset.id)setSelected(undefined);}}><Icon name="delete"/></button></div></td>
  </tr>}</For></tbody></table></div>
  <Show when={current()}>{preset=><fieldset class={table.editor} disabled={props.disabled}>
   <legend>{t('snooze-edit')}</legend><div class={table.headerFields}>
    <label class={css.field}>{t('snooze-name')}<input class={css.input} maxlength={80} value={preset().label??''} onInput={event=>{const label=event.currentTarget.value;props.onChange(props.value.map(p=>{if(p.id!==selected())return p;const {label:_label,...rest}=p;return label?{...rest,label}:rest;}));}}/></label>
    <label class={css.field}>{t('snooze-rule')}<select class={css.select} value={preset().kind} onChange={event=>kind(event.currentTarget.value as SnoozeKind)}><For each={SNOOZE_KINDS}>{value=><option value={value}>{t(`snooze-rule-${value}`)}</option>}</For></select></label>
    <Show when={preset().kind==='hours'||preset().kind==='days'}><label class={css.field}>{t('snooze-amount')}<input class={css.input} type="number" min={1} max={preset().kind==='hours'?87600:3650} step={1} value={preset().amount??0} onInput={event=>patch({amount:Number(event.currentTarget.value)})}/></label></Show>
    <Show when={preset().kind==='weekday'||preset().kind==='nth-weekday'}><label class={css.field}>{t('snooze-weekday')}<select class={css.select} value={preset().weekday??1} onChange={event=>patch({weekday:Number(event.currentTarget.value)})}><For each={[0,1,2,3,4,5,6]}>{day=><option value={day}>{scheduleWeekday(day)}</option>}</For></select></label></Show>
    <Show when={preset().kind==='nth-weekday'}><label class={css.field}>{t('snooze-occurrence')}<select class={css.select} value={preset().ordinal??1} onChange={event=>patch({ordinal:Number(event.currentTarget.value)})}><For each={[1,2,3,4,5,-1]}>{ordinal=><option value={ordinal}>{ordinal===-1?t('snooze-last'):String(ordinal)}</option>}</For></select></label></Show>
    <Show when={preset().kind==='day-of-month'}><label class={css.field}>{t('snooze-month-day')}<input class={css.input} type="number" min={1} max={31} step={1} value={preset().day??1} onInput={event=>patch({day:Number(event.currentTarget.value)})}/></label></Show>
    <Show when={preset().kind!=='hours'}><label class={css.field}>{t('snooze-time')}<input class={css.input} type="time" required value={preset().time??'09:00'} onInput={event=>patch({time:event.currentTarget.value})}/></label></Show>
   </div><p class={css.meta}>{t('snooze-calendar-help')}</p><p class={css.meta}>{t('snooze-calendar-advanced-help')}</p>
  </fieldset>}</Show>
 </div>;
}
