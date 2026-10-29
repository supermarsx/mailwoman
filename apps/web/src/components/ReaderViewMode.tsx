// The reader's view-mode dropdown (t57): ONE compact button in the action row
// showing the current opening mode (Full / No media / Plain text) as an icon
// with a chevron, which opens a small themed menu of the three modes. It
// replaces the segmented MaxSecuritySwitch; the store behind it is the same
// (max-security.ts: per-sender choice, global default, admin floor), so modes
// below the admin floor are listed but disabled and can never be chosen.
// Keyboard comes from ContextMenu: Enter/Space on the button opens it with the
// first item focused, arrows move, Enter chooses, Escape closes back to it.

import {createEffect,type JSX} from 'solid-js';
import {createContextMenu,type ContextAction} from './ContextMenu.tsx';
import {Icon,type IconName} from './Icon.tsx';
import {SECURITY_MODES,SECURITY_MODE_LABELS,SECURITY_MODE_HINTS,isAtLeastAsStrict,type SecurityMode} from '../viewers/max-security.ts';
import * as a11y from './mailA11y.css.ts';
import {t} from '../i18n/index.ts';
import './reader-action.css';

export const VIEW_MODE_ICON:Record<SecurityMode,IconName>={'full-sanitized':'image','sanitized-no-media':'eyeOff','plain-text':'paragraph'};

export interface ViewModeProps {value:SecurityMode;floor:SecurityMode|null;onChange:(mode:SecurityMode)=>void}

/** The three modes as menu entries; used by the dropdown and by the action
 *  row's More menu when the dropdown is folded. */
export function viewModeActions(props:ViewModeProps,separatorBefore=false):ContextAction[] {
 return SECURITY_MODES.map((mode,index)=>({id:`view:${mode}`,label:separatorBefore?t('mail-maxsec-view',{mode:SECURITY_MODE_LABELS[mode]()}):SECURITY_MODE_LABELS[mode](),icon:VIEW_MODE_ICON[mode],checked:props.value===mode,current:props.value===mode,disabled:props.floor!==null&&!isAtLeastAsStrict(mode,props.floor),separatorBefore:separatorBefore&&index===0,run:()=>props.onChange(mode)}));
}

export function ReaderViewMode(props:ViewModeProps&{scope:string}):JSX.Element {
 let button!:HTMLButtonElement;
 const menu=createContextMenu({label:()=>t('mail-maxsec-menu'),scope:()=>props.scope,actions:()=>viewModeActions(props),restoreFocus:()=>button?.focus()});
 // The menu closes on the pointerdown that lands on this button; the click
 // that follows must not open it again.
 let closedAt=0;
 createEffect(()=>{if(!menu.isOpen())closedAt=performance.now();});
 const label=()=>t('mail-maxsec-view',{mode:SECURITY_MODE_LABELS[props.value]()});
 function toggle(event:MouseEvent):void {if(menu.isOpen()){menu.close();return;}if(event.detail>0&&performance.now()-closedAt<250)return;menu.open(event);}
 return <>
  <button ref={button} type="button" class={`btn btn--ghost reader-action reader-action--compact reader-view-mode ${a11y.focusable}`} aria-label={label()} title={`${label()}. ${SECURITY_MODE_HINTS[props.value]()}`} aria-haspopup="menu" aria-expanded={menu.isOpen()} data-testid="max-security-switch" data-mode={props.value} onClick={toggle}><Icon name={VIEW_MODE_ICON[props.value]}/><span class="reader-view-mode__chevron" aria-hidden="true"><Icon name="chevronDown"/></span></button>
  {menu.menu()}
 </>;
}
