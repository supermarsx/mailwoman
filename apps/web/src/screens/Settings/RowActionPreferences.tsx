import {rowReset as inheritLink} from './layout/layout.css.ts';
import {For,Show,type JSX} from 'solid-js';
import {ROW_ACTION_IDS,ROW_ACTION_LABELS,ROW_ACTION_LAYOUT_KEY,DEFAULT_ROW_ACTION_LAYOUT,isRowActionLayout,type RowActionId} from '../../state/rowActionLayout.ts';
import {Icon} from '../../components/Icon.tsx';
import {t,loadCatalog} from '../../i18n';
import {useSetting} from './useSetting.ts';
import {SettingsCard} from './layout/SettingsCard.tsx';
import {LockNote} from './LockNote.tsx';
import * as css from './styles.css.ts';
import * as compact from './readerActionPreferences.css.ts';
void loadCatalog('settings');void loadCatalog('mail');void loadCatalog('reader-actions');
export function RowActionPreferences():JSX.Element{
 const handle=useSetting<string[]>(ROW_ACTION_LAYOUT_KEY,DEFAULT_ROW_ACTION_LAYOUT,isRowActionLayout);
 const disabled=()=>!handle.entry()||handle.locked()||handle.saving();
 const placement=(id:RowActionId)=>handle.value().find(value=>value.endsWith(`:${id}`))?.split(':')[0]??'hidden';
 const rows=()=>[...handle.value().map(value=>value.split(':')[1] as RowActionId),...ROW_ACTION_IDS.filter(id=>placement(id)==='hidden')];
 const place=(id:RowActionId,value:string)=>{const next=handle.value().filter(item=>!item.endsWith(`:${id}`));if(value!=='hidden')next.push(`${value}:${id}`);void handle.set(next);};
 const move=(id:RowActionId,delta:number)=>{const items=[...handle.value()],index=items.findIndex(value=>value.endsWith(`:${id}`)),target=index+delta;if(index<0||target<0||target>=items.length)return;[items[index],items[target]]=[items[target]!,items[index]!];void handle.set(items);};
 return <SettingsCard id="settings-card-row-actions" title={t('settings-row-actions-custom-title')} description={t('settings-row-actions-custom-help')}><LockNote locked={handle.locked()}/><div class={compact.list}><For each={rows()}>{id=><div class={compact.row} data-row-action={id}><Icon name={id==='smart-archive'?'smartArchive':id==='view-draft'?'eye':id==='label'?'tag':id==='flag'?'flag':id}/><label class={compact.label} for={`quick-action-${id}`}>{t(ROW_ACTION_LABELS[id])}</label><select class={compact.select} id={`quick-action-${id}`} disabled={disabled()} value={placement(id)} onChange={event=>place(id,event.currentTarget.value)}><option value="quick">{t('reader-actions-option-toolbar')}</option><option value="more">{t('reader-actions-option-menu')}</option><option value="hidden">{t('reader-actions-option-hidden')}</option></select><span class={compact.order}><button type="button" class={css.ghost} aria-label={t('settings-row-action-up',{name:t(ROW_ACTION_LABELS[id])})} disabled={disabled()||placement(id)==='hidden'||handle.value().findIndex(value=>value.endsWith(`:${id}`))===0} onClick={()=>move(id,-1)}><Icon name="chevronUp"/></button><button type="button" class={css.ghost} aria-label={t('settings-row-action-down',{name:t(ROW_ACTION_LABELS[id])})} disabled={disabled()||placement(id)==='hidden'||handle.value().findIndex(value=>value.endsWith(`:${id}`))===handle.value().length-1} onClick={()=>move(id,1)}><Icon name="chevronDown"/></button></span></div>}</For></div><button type="button" class={inheritLink} disabled={disabled()} onClick={()=>void handle.reset()}>{t('reader-inherit')}</button><Show when={handle.error()}><p role="alert">{handle.error()}</p></Show></SettingsCard>;
}
