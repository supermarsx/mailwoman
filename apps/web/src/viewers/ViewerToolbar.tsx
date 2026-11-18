import './viewer-toolbar.css';
import {useCommandRibbon} from '../components/CommandRibbonContext.tsx';
import {InRibbonSlot} from '../components/compose/ribbonParts.tsx';
import {RibbonGroup,RibbonItem} from '../components/bars/index.ts';
import {Index,Show,type JSX} from 'solid-js';
import {Icon,type IconName} from '../components/Icon.tsx';
import {createContextMenu} from '../components/ContextMenu.tsx';
import {previewToolPlacement} from './filePreviewPreferences.ts';
import {attachmentActionStyle} from './previewPreferences.ts';
import {loadCatalog,t} from '../i18n';
void loadCatalog('file-previews');
export interface ViewerToolAction{id:string;label:string;icon:IconName;run:()=>void|Promise<void>;disabled?:boolean;pressed?:boolean|undefined;testId?:string;render?:()=>JSX.Element}
export function ViewerToolbar(props:{actions:ViewerToolAction[];scope:string;general?:boolean}):JSX.Element{
 const ribbon=useCommandRibbon();
 const centralized=()=>ribbon?.enabled()===true&&ribbon.active()&&ribbon.preview()!==undefined;
 const groupLabel=()=>props.general?t('preview-common'):t(({image:'preview-kind-image',text:'preview-kind-text',html:'preview-kind-html',pdf:'preview-kind-pdf',media:'preview-media'} as Record<string,string>)[props.actions[0]?.id.split('.')[0]??'']??'preview-tools-title');
 const shown=()=>props.actions.filter(action=>previewToolPlacement(action.id)==='toolbar'&&!(centralized()&&!props.general&&action.id==='download'));
 const overflow=()=>props.actions.filter(action=>previewToolPlacement(action.id)==='menu'&&!(centralized()&&!props.general&&action.id==='download'));
 const menu=createContextMenu({label:()=>t('mail-more-actions'),scope:()=>props.scope,actions:()=>overflow().map(action=>({id:action.id,label:action.label,icon:action.icon,disabled:action.disabled===true,...(action.pressed!==undefined?{checked:action.pressed}:{}),run:action.run}))});
 const controls=()=><><Index each={shown()}>{action=><ToolSlot ribbon={centralized()}><Show when={action().render} fallback={<button type="button" class="btn btn--ghost" title={action().label} aria-label={action().label} aria-pressed={action().pressed} disabled={action().disabled} data-testid={action().testId} onClick={()=>void action().run()}><Show when={attachmentActionStyle()!=='labels'}><Icon name={action().icon}/></Show><Show when={attachmentActionStyle()!=='icons'}>{action().label}</Show></button>}>{render=>render()()}</Show></ToolSlot>}</Index><Show when={overflow().length}><ToolSlot ribbon={centralized()}><button type="button" class="btn btn--ghost" aria-label={t('mail-more-actions')} aria-haspopup="menu" aria-expanded={menu.isOpen()} onClick={event=>menu.toggleBelow(event.currentTarget)}><Icon name="more"/></button></ToolSlot></Show>{menu.menu()}</>;
 return <Show when={centralized()} fallback={controls()}><Show when={ribbon?.fileTarget()}>{target=><InRibbonSlot target={target()}><RibbonGroup label={groupLabel()} icon="preferences" priority={20}>{controls()}</RibbonGroup></InRibbonSlot>}</Show></Show>;
}
function ToolSlot(props:{ribbon:boolean;children:JSX.Element}):JSX.Element{return <Show when={props.ribbon} fallback={props.children}><RibbonItem>{props.children}</RibbonItem></Show>;}

export function downloadPreview(url:string,name:string):void{const link=document.createElement('a');link.href=url;link.download=name.replace(/[\\/]/g,'_');link.rel='noopener';link.click();}

/** Search fields and status readouts follow their viewer's buttons into the ribbon. */
export function PreviewRibbonExtras(props:{children:JSX.Element;large?:boolean;label?:string}):JSX.Element {
 const ribbon=useCommandRibbon();
 const centralized=()=>ribbon?.enabled()===true&&ribbon.active()&&ribbon.preview()!==undefined;
 return <Show when={centralized()} fallback={props.children}><Show when={ribbon?.fileTarget()}>{target=><InRibbonSlot target={target()}><RibbonGroup label={props.label??t('preview-tools-title')} icon="info" priority={15}><RibbonItem size={props.large?'large':'small'} {...(props.large?{inlineSize:'min(24rem, calc(100vw - 3rem))'}:{})}><div class="file-ribbon-extras">{props.children}</div></RibbonItem></RibbonGroup></InRibbonSlot>}</Show></Show>;
}
