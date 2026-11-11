// Presentation preferences never grant a message feature or bypass its policy.
// Toolbar choices still use responsive overflow; menu choices go straight to More.
import {settings} from '../api/settings.ts';
export const READER_ACTION_PLACEMENTS=['toolbar','menu','hidden'] as const;
export type ReaderActionPlacement=typeof READER_ACTION_PLACEMENTS[number];
export const READER_ACTIONS=[
 {id:'reply',labelKey:'reader-action-reply',defaultPlacement:'toolbar'},
 {id:'reply-all',labelKey:'reader-action-reply-all',defaultPlacement:'toolbar'},
 {id:'forward',labelKey:'reader-action-forward',defaultPlacement:'toolbar'},
 {id:'edit-draft',labelKey:'reader-action-edit-draft',defaultPlacement:'toolbar'},
 {id:'open-draft',labelKey:'reader-action-open-draft',defaultPlacement:'toolbar'},
 {id:'delete',labelKey:'reader-action-delete',defaultPlacement:'toolbar'},
 {id:'delete-sender',labelKey:'reader-action-delete-sender',defaultPlacement:'hidden'},
 {id:'finish-followup',labelKey:'reader-action-finish-followup',defaultPlacement:'menu'},
 {id:'archive',labelKey:'reader-action-archive',defaultPlacement:'toolbar'},
 {id:'move',labelKey:'reader-action-move',defaultPlacement:'toolbar'},
 {id:'conversation',labelKey:'reader-action-conversation',defaultPlacement:'toolbar'},
 {id:'links',labelKey:'reader-action-links',defaultPlacement:'toolbar'},
 {id:'read',labelKey:'reader-action-read',defaultPlacement:'toolbar'},
 {id:'flag',labelKey:'reader-action-flag',defaultPlacement:'toolbar'},
 {id:'pin',labelKey:'reader-action-pin',defaultPlacement:'toolbar'},
 {id:'spam',labelKey:'reader-action-spam',defaultPlacement:'toolbar'},
 {id:'security',labelKey:'reader-action-security',defaultPlacement:'toolbar'},
 {id:'headers',labelKey:'reader-action-headers',defaultPlacement:'toolbar'},
 {id:'sweep',labelKey:'reader-action-sweep',defaultPlacement:'toolbar'},
 {id:'print',labelKey:'reader-action-print',defaultPlacement:'toolbar'},
 {id:'export',labelKey:'reader-action-export',defaultPlacement:'toolbar'},
 {id:'properties',labelKey:'reader-action-properties',defaultPlacement:'toolbar'},
 {id:'zoom',labelKey:'reader-action-zoom',defaultPlacement:'toolbar'},
 {id:'search',labelKey:'reader-action-search',defaultPlacement:'toolbar'},
 {id:'colors',labelKey:'reader-action-colors',defaultPlacement:'toolbar'},
 {id:'save-sender',labelKey:'reader-action-save-sender',defaultPlacement:'menu'},
 {id:'save-contacts',labelKey:'reader-action-save-contacts',defaultPlacement:'menu'},
 {id:'focus-focused',labelKey:'reader-action-focus-focused',defaultPlacement:'menu'},
 {id:'focus-other',labelKey:'reader-action-focus-other',defaultPlacement:'menu'},
 {id:'not-spam',labelKey:'reader-action-not-spam',defaultPlacement:'menu'},
 {id:'view-source',labelKey:'reader-action-view-source',defaultPlacement:'menu'},
 {id:'create-filter',labelKey:'reader-action-create-filter',defaultPlacement:'menu'},
 {id:'create-task',labelKey:'message-task-action',defaultPlacement:'menu'},
 {id:'block-sender',labelKey:'reader-action-block-sender',defaultPlacement:'menu'},
 ] as const;
export type ReaderActionId=typeof READER_ACTIONS[number]['id'];
export const readerActionKey=(id:ReaderActionId):`reader.actions.${ReaderActionId}`=>`reader.actions.${id}`;
export function isReaderActionPlacement(value:unknown):value is ReaderActionPlacement {return READER_ACTION_PLACEMENTS.some(item=>item===value);}
export function readerActionPlacement(id:string,map:Record<string,{value:unknown}|undefined>=settings()):ReaderActionPlacement {
 const definition=READER_ACTIONS.find(item=>item.id===id);
 // New menu actions remain reachable until they gain a registered preference.
 if(!definition)return 'menu';
 const value=map[readerActionKey(definition.id)]?.value;
 return isReaderActionPlacement(value)?value:definition.defaultPlacement;
}
/** Context menu IDs remain unchanged; persisted keys use registry-safe IDs. */
export function readerMenuActionId(id:string):ReaderActionId|undefined {
 const stable=id==='focus:focused'?'focus-focused':id==='focus:other'?'focus-other':id;
 return READER_ACTIONS.find(item=>item.defaultPlacement==='menu'&&item.id===stable)?.id;
}
