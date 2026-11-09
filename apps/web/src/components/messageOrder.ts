// One effective message-order preference drives the list bar and context menus.
import {createSignal} from 'solid-js';
import {settings,writeSetting} from '../api/settings.ts';
import {t} from '../i18n';
import type {IconName} from './Icon.tsx';
import type {ContextAction} from './ContextMenu.tsx';
export const MESSAGE_LIST_ORDERS=['newest','oldest','sender','sender-desc','subject','subject-desc','sent-newest','sent-oldest','importance','importance-asc','size','size-asc'] as const;
export type MessageListOrder=typeof MESSAGE_LIST_ORDERS[number];
export const MESSAGE_LIST_GROUPS=['none','received','sent','sender','subject','importance','size'] as const;
export type MessageListGroup=typeof MESSAGE_LIST_GROUPS[number];
export const MESSAGE_LIST_GROUP_KEY='mail.messageListGroup';
export function messageListGroup():MessageListGroup {const value=settings()[MESSAGE_LIST_GROUP_KEY]?.value;return MESSAGE_LIST_GROUPS.some(group=>group===value)?value as MessageListGroup:'none';}
export const orderIcons:Record<MessageListOrder,IconName>={newest:'inbox',oldest:'clock',sender:'person','sender-desc':'person',subject:'sortAlphaAsc','subject-desc':'sortAlphaDesc','sent-newest':'sent','sent-oldest':'sent',importance:'flag','importance-asc':'flag',size:'package','size-asc':'package'};
export const groupIcons:Record<MessageListGroup,IconName>={none:'minus',received:'inbox',sent:'sent',sender:'person',subject:'heading',importance:'flag',size:'package'};
export const MESSAGE_LIST_ORDER_KEY='mail.messageListOrder';
export function isMessageListOrder(value:unknown):value is MessageListOrder{return MESSAGE_LIST_ORDERS.some(order=>order===value);}
export function messageListOrder():MessageListOrder {const value=settings()[MESSAGE_LIST_ORDER_KEY]?.value;return isMessageListOrder(value)?value:'newest';}
export function createMessageOrder(onError:()=>void){
 const [saving,setSaving]=createSignal(false);
 const locked=()=>settings()[MESSAGE_LIST_ORDER_KEY]?.locked===true;
 async function set(value:MessageListOrder):Promise<boolean>{
  if(locked()){onError();return false;}if(saving())return false;if(value===messageListOrder())return true;
  setSaving(true);
  try{await writeSetting(MESSAGE_LIST_ORDER_KEY,value);return true;}catch{onError();return false;}finally{setSaving(false);}
 }
 const groupLocked=()=>settings()[MESSAGE_LIST_GROUP_KEY]?.locked===true;
 async function setGroup(value:MessageListGroup):Promise<boolean>{
  if(groupLocked()){onError();return false;}if(saving())return false;if(value===messageListGroup())return true;setSaving(true);
  try{await writeSetting(MESSAGE_LIST_GROUP_KEY,value);return true;}catch{onError();return false;}finally{setSaving(false);}
 }
 const sortActions=():ContextAction[]=>MESSAGE_LIST_ORDERS.map(value=>({id:`message-order:${value}`,label:t(`mail-list-order-${value}`),icon:orderIcons[value],radio:true,checked:messageListOrder()===value,disabled:locked()||saving(),...(locked()?{title:t('mail-list-order-locked')}:{}),run:async()=>{await set(value);}}));
 const actions=():ContextAction[]=>[...sortActions(),{id:'message-group',label:t('mail-group-by'),icon:'shapes',separatorBefore:true,children:MESSAGE_LIST_GROUPS.map(value=>({id:`message-group:${value}`,label:t(`mail-list-group-${value}`),icon:groupIcons[value],radio:true,checked:messageListGroup()===value,disabled:groupLocked()||saving(),...(groupLocked()?{title:t('mail-list-group-locked')}:{}),run:async()=>{await setGroup(value);}})),run:()=>{}}];
 const contextAction=():ContextAction=>({id:'message-order',label:t('mail-order-by'),icon:'sort',separatorBefore:true,disabled:saving(),children:actions(),run:()=>{}});
 return {value:messageListOrder,group:messageListGroup,locked,groupLocked,saving,set,setGroup,actions,contextAction};
}
export type MessageOrderController=ReturnType<typeof createMessageOrder>;
