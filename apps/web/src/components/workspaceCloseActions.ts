import type {SubTab} from '../realtime/subTabs.ts';
import {loadCatalog,t} from '../i18n/index.ts';
void loadCatalog('workspace-tabs');
import type {ContextAction} from './ContextMenu.tsx';
/** Shared closing choices for the tab strip and visible split panes. */
export function workspaceCloseAction(options:{tabs:()=>readonly SubTab[];id:()=>string;canClose:(id:string)=>boolean;rtl:()=>boolean;close:()=>void;closeMany:(ids:string[])=>void}):ContextAction {
 const sides=()=>{
  const tabs=options.tabs(),index=tabs.findIndex(tab=>tab.id===options.id());
  if(index<0)return {others:[],left:[],right:[],all:[]} as Record<'others'|'left'|'right'|'all',string[]>;
  const eligible=(tab:SubTab)=>!tab.pinned&&options.canClose(tab.id);
  const before=tabs.slice(0,index).filter(eligible).map(tab=>tab.id),after=tabs.slice(index+1).filter(eligible).map(tab=>tab.id);
  return {others:[...before,...after],left:options.rtl()?after:before,right:options.rtl()?before:after,all:tabs.filter(eligible).map(tab=>tab.id)};
 };
 const tab=options.tabs().find(tab=>tab.id===options.id()),available=sides();
 const children:ContextAction[]=[];
 if(tab&&options.canClose(tab.id))children.push({id:'close-current',label:t('mail-subtab-close',{title:tab.title}),icon:'close',run:()=>{if(options.tabs().some(tab=>tab.id===options.id())&&options.canClose(options.id()))options.close();}});
 for(const key of ['others','left','right','all'] as const)children.push({id:`close-${key}`,label:t(`workspace-tabs-close-${key}`),icon:'close',disabled:available[key].length===0,run:()=>{const ids=sides()[key];if(ids.length)options.closeMany(ids);}});
 return {id:'close',label:t('common-close'),icon:'close',disabled:children.every(child=>child.disabled),children,run:()=>{}};
}
