// The reader's per-message colour mode (a menu button in the action row).
//
// It lists the same modes as Settings > Reader > Message colors and applies a
// choice to the message on screen only; the saved setting is untouched unless
// "Make this my default" is confirmed. Folded into More, the same entries form a
// submenu (`colorRowAction`). An admin-locked `reader.colorMode` disables it.

import {onCleanup,onMount,type JSX} from 'solid-js';
import {createContextMenu,type ContextAction} from './ContextMenu.tsx';
import {Icon} from './Icon.tsx';
import {READER_COLOR_CHOICES,type ReaderColorChoice} from '../viewers/readerColorMode.ts';
import {loadCatalog,t} from '../i18n/index.ts';
import * as a11y from './mailA11y.css.ts';
import './reader-action.css';

export interface ReaderColorProps {
 /** What the message shows now (an override, else the saved setting). */
 value:ReaderColorChoice;
 /** The user's saved setting. */
 setting:ReaderColorChoice;
 locked:boolean;
 /** `null` goes back to the saved setting. */
 onChange:(choice:ReaderColorChoice|null)=>void;
 onMakeDefault:(choice:ReaderColorChoice)=>void;
}

export const colorLabel=(choice:ReaderColorChoice):string=>t(`reader-option-${choice}`);

export function colorActions(props:ReaderColorProps):ContextAction[] {
 const lock=props.locked?t('mail-color-locked'):undefined;
 const base=(choice:ReaderColorChoice):ContextAction=>({id:`color:${choice}`,label:colorLabel(choice),icon:'palette',checked:props.value===choice,current:props.value===choice,radio:true,disabled:props.locked,...(lock?{title:lock}:{}),run:()=>props.onChange(choice)});
 return [
  ...READER_COLOR_CHOICES.map(base),
  {id:'color:reset',label:t('mail-color-use-default',{mode:colorLabel(props.setting)}),icon:'undo',separatorBefore:true,disabled:props.locked||props.value===props.setting,...(lock?{title:lock}:{}),run:()=>props.onChange(null)},
  {id:'color:make-default',label:t('mail-color-make-default'),icon:'check',disabled:props.locked||props.value===props.setting,...(lock?{title:lock}:{}),run:()=>props.onMakeDefault(props.value)},
 ];
}

/** The same entries as one "Message colors" submenu of the More menu. */
export function colorRowAction(props:()=>ReaderColorProps,scope:()=>string):{id:string;inline:()=>JSX.Element;menu:()=>ContextAction[]} {
 return {id:'colors',inline:()=><ReaderColorMenu {...props()} scope={scope()} />,menu:()=>[{id:'colors',label:t('mail-color-menu'),icon:'palette',children:colorActions(props()),run:()=>{}}]};
}

export function ReaderColorMenu(props:ReaderColorProps&{scope:string}):JSX.Element {
 let button!:HTMLButtonElement;
 onMount(()=>void loadCatalog('reader'));
 const menu=createContextMenu({label:()=>t('mail-color-menu'),scope:()=>props.scope,actions:()=>colorActions(props),restoreFocus:()=>button?.focus()});
 let reopenGuard=false;
 onMount(()=>{const press=(event:PointerEvent):void=>{reopenGuard=menu.isOpen()&&event.target instanceof Node&&button.contains(event.target);};document.addEventListener('pointerdown',press,true);onCleanup(()=>document.removeEventListener('pointerdown',press,true));});
 const label=()=>t('mail-color-button',{mode:colorLabel(props.value)});
 function toggle(event:MouseEvent):void {if(props.locked)return;if(menu.isOpen()){menu.close();return;}if(reopenGuard){reopenGuard=false;return;}menu.open(event);}
 return <>
  <button ref={button} type="button" class={`btn btn--ghost reader-action reader-action--compact reader-color-mode ${a11y.focusable}`} classList={{'reader-color-mode--overridden':props.value!==props.setting}} aria-label={label()} title={props.locked?t('mail-color-locked'):label()} aria-haspopup="menu" aria-expanded={menu.isOpen()} aria-disabled={props.locked?'true':undefined} data-testid="reader-color-mode" data-mode={props.value} onClick={toggle}><Icon name="palette"/><span class="reader-view-mode__chevron" aria-hidden="true"><Icon name="chevronDown"/></span></button>
  {menu.menu()}
 </>;
}
