// One place that owns the open Properties dialog. Menus, the Settings folder list
// and the Alt+Enter shortcut call `openFolderProperties` / `openMessageProperties`;
// `PropertiesHost` (mounted once in the mail screen) draws the dialog. A request
// is dropped when the administrator has switched that dialog off.
import {Show,createSignal,lazy,type JSX} from 'solid-js';
import type {Email,Mailbox} from '../../api/jmap-types.ts';
import type {FolderDialogActions} from './FolderProperties.tsx';
import type {MessageDialogActions} from './MessageProperties.tsx';
import {folderDialogEnabled,matchesShortcut,messageDialogEnabled,shortcutEnabled,shortcutKey} from './policy.ts';
import {preferenceAccount} from '../../api/accountPreferenceLifecycle.ts';
import {loadCatalog} from '../../i18n';

void loadCatalog('properties');
// The dialogs are loaded when first opened, so they stay out of the entry bundle.
const FolderPropertiesDialog=lazy(async()=>({default:(await import('./FolderProperties.tsx')).FolderPropertiesDialog}));
const MessageDialogs=lazy(async()=>({default:(await import('./MessageProperties.tsx')).MessageDialogs}));

type Open={kind:'folder';box:Mailbox;actions?:FolderDialogActions;owner:string|null}|{kind:'message';email:Email;actions?:MessageDialogActions;owner:string|null}|null;
const [open,setOpen]=createSignal<Open>(null);

export function openFolderProperties(box:Mailbox,actions?:FolderDialogActions):boolean {
 if(!folderDialogEnabled())return false;
 setOpen({kind:'folder',box,...(actions?{actions}:{}),owner:preferenceAccount()?.id??null});return true;
}
export function openMessageProperties(email:Email,actions?:MessageDialogActions):boolean {
 if(!messageDialogEnabled())return false;
 setOpen({kind:'message',email,...(actions?{actions}:{}),owner:preferenceAccount()?.id??null});return true;
}
export function closeProperties():void {setOpen(null);}
export function propertiesOpen():'folder'|'message'|null {return open()?.kind??null;}

/** Whether a key event should open Properties for the focused folder or selected message; the caller decides which. */
export function isPropertiesShortcut(event:KeyboardEvent):boolean {return shortcutEnabled()&&matchesShortcut(event,shortcutKey());}

export function PropertiesHost():JSX.Element {
 return <>
  <Show when={open()}>{current=>{
   const value=current();
   // A dialog never outlives the account it was opened for.
   if(value.owner!==(preferenceAccount()?.id??null))return null;
   return value.kind==='folder'
    ?<FolderPropertiesDialog box={value.box} {...(value.actions?{actions:value.actions}:{})} onClose={closeProperties}/>
    :<MessageDialogs email={value.email} {...(value.actions?{actions:value.actions}:{})} onClose={closeProperties}/>;
  }}</Show>
 </>;
}
