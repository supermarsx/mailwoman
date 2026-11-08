// One effective message-order preference drives the list bar and context menus.
import {createSignal} from 'solid-js';
import {settings,writeSetting} from '../api/settings.ts';
import {t} from '../i18n';
import type {ContextAction} from './ContextMenu.tsx';
export const MESSAGE_LIST_ORDERS=['newest','oldest','sender','subject'] as const;
export type MessageListOrder=typeof MESSAGE_LIST_ORDERS[number];
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
 const actions=():ContextAction[]=>MESSAGE_LIST_ORDERS.map(value=>({id:`message-order:${value}`,label:t(`mail-list-order-${value}`),icon:'sort',radio:true,checked:messageListOrder()===value,disabled:locked()||saving(),...(locked()?{title:t('mail-list-order-locked')}:{}),run:async()=>{await set(value);}}));
 const contextAction=():ContextAction=>({id:'message-order',label:t('mail-order-by'),icon:'sort',separatorBefore:true,disabled:locked()||saving(),...(locked()?{title:t('mail-list-order-locked')}:{}),children:actions(),run:()=>{}});
 return {value:messageListOrder,locked,saving,set,actions,contextAction};
}
export type MessageOrderController=ReturnType<typeof createMessageOrder>;
