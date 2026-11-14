import {Index,Show,type JSX} from 'solid-js';
import {Icon,type IconName} from '../components/Icon.tsx';
import {createContextMenu} from '../components/ContextMenu.tsx';
import {previewToolPlacement} from './filePreviewPreferences.ts';
import {attachmentActionStyle} from './previewPreferences.ts';
import {t} from '../i18n';
export interface ViewerToolAction{id:string;label:string;icon:IconName;run:()=>void|Promise<void>;disabled?:boolean;pressed?:boolean|undefined;testId?:string;render?:()=>JSX.Element}
export function ViewerToolbar(props:{actions:ViewerToolAction[];scope:string}):JSX.Element{
 const shown=()=>props.actions.filter(action=>previewToolPlacement(action.id)==='toolbar');
 const overflow=()=>props.actions.filter(action=>previewToolPlacement(action.id)==='menu');
 const menu=createContextMenu({label:()=>t('mail-more-actions'),scope:()=>props.scope,actions:()=>overflow().map(action=>({id:action.id,label:action.label,icon:action.icon,disabled:action.disabled===true,...(action.pressed!==undefined?{checked:action.pressed}:{}),run:action.run}))});
 return <><Index each={shown()}>{action=><Show when={action().render} fallback={<button type="button" class="btn btn--ghost" title={action().label} aria-label={action().label} aria-pressed={action().pressed} disabled={action().disabled} data-testid={action().testId} onClick={()=>void action().run()}><Show when={attachmentActionStyle()!=='labels'}><Icon name={action().icon}/></Show><Show when={attachmentActionStyle()!=='icons'}>{action().label}</Show></button>}>{render=>render()()}</Show>}</Index><Show when={overflow().length}><button type="button" class="btn btn--ghost" aria-label={t('mail-more-actions')} aria-haspopup="menu" aria-expanded={menu.isOpen()} onClick={event=>menu.openBelow(event.currentTarget)}><Icon name="more"/></button></Show>{menu.menu()}</>;
}
export function downloadPreview(url:string,name:string):void{const link=document.createElement('a');link.href=url;link.download=name.replace(/[\\/]/g,'_');link.rel='noopener';link.click();}
