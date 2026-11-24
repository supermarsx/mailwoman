import {createSignal} from 'solid-js';
import {settings} from '../api/settings.ts';
import {accountUiPreferences,setAccountUiPreferences} from '../api/accountPreferences.ts';
import {allowedOverrides} from './properties/policy.ts';
import {withOverride} from './properties/folderOverrides.ts';

export const VIEW_COLUMNS=['sender','subject','received','sent','to','size','importance'] as const;
export type ViewColumn=typeof VIEW_COLUMNS[number];
export interface FolderViewPreferences {
 columns?:ViewColumn[];columnWidths?:Partial<Record<ViewColumn,number>>;
 sort?:string;group?:string;filterQuery?:string;filter?:'all'|'unread'|'read'|'flagged'|'attachments'|'mentions';
 font?:'inherit'|'sans'|'serif'|'mono';fontSize?:number;rowSpacing?:number;
 highlight?:'default'|'unread'|'flagged'|'important'|'none';
 readingPane?:'right'|'bottom'|'off';listWidth?:number;listHeight?:number;style?:'cards'|'table';
}
export const VIEW_FIELDS={
 columns:{key:'mail.messageListColumns',fallback:['sender','subject','received']},
 sort:{key:'mail.messageListOrder',fallback:'newest'},group:{key:'mail.messageListGroup',fallback:'none'},
 filterQuery:{key:'mail.messageListFilterQuery',fallback:''},filter:{key:'mail.messageListFilter',fallback:'all'},font:{key:'mail.messageListFont',fallback:'inherit'},
 fontSize:{key:'mail.messageListFontSize',fallback:14},rowSpacing:{key:'mail.messageListRowSpacing',fallback:0},
 highlight:{key:'mail.messageListHighlight',fallback:'default'},readingPane:{key:'mail.readingPane',fallback:'right'},
 listWidth:{key:'mail.messageListWidth',fallback:340},listHeight:{key:'mail.messageListHeight',fallback:320},
 style:{key:'mail.messageListStyle',fallback:'cards'},
} as const;
export type ViewField=keyof typeof VIEW_FIELDS;
export const [activeFolderViewId,setActiveFolderViewId]=createSignal<string|undefined>();
export function folderViewAllowed():boolean{return allowedOverrides().includes('view');}
export function folderViewLocked(field:ViewField):boolean {return settings()[VIEW_FIELDS[field].key]?.locked===true;}
export function folderViewValue<K extends ViewField>(field:K,id=activeFolderViewId()):NonNullable<FolderViewPreferences[K]> {
 const def=VIEW_FIELDS[field],entry=settings()[def.key],prefs=accountUiPreferences();
 let value:unknown=entry?.value??def.fallback;
 // Preserve pre-existing account layout choices while allowing enforced policy.
 if(!entry?.locked&&entry?.source!=='user'){
  if(field==='readingPane')value=prefs.readingPane??value;
  if(field==='listWidth')value=prefs.messageListWidth??value;
  if(field==='listHeight')value=prefs.messageListHeight??value;
 }
 const own=id&&allowedOverrides().includes('view')?prefs.folderProps?.[id]?.view?.[field]:undefined;
 if(!entry?.locked&&own!==undefined)value=own;
 if(field==='columns'){const columns=Array.isArray(value)?[...new Set(value.filter(c=>VIEW_COLUMNS.includes(c)))]:[];value=columns.length?columns:VIEW_FIELDS.columns.fallback;}
 return value as NonNullable<FolderViewPreferences[K]>;
}
export function folderColumnWidth(column:ViewColumn,id=activeFolderViewId()):number {
 const entry=settings()[`mail.messageListColumnWidth.${column}`];
 const own=id&&allowedOverrides().includes('view')?accountUiPreferences().folderProps?.[id]?.view?.columnWidths?.[column]:undefined;
 return Math.max(64,Math.min(800,Number(!entry?.locked&&own!==undefined?own:entry?.value??(column==='subject'?300:column==='size'||column==='importance'?90:180))));
}
export function setFolderView(id:string,patch:{[K in keyof FolderViewPreferences]?:FolderViewPreferences[K]|undefined}):void {
 if(!allowedOverrides().includes('view'))return;
 const all=accountUiPreferences().folderProps,own=all?.[id]?.view??{};
 const next={...own};
 for(const field of Object.keys(patch) as (keyof FolderViewPreferences)[]){
  if(field==='columnWidths'){
   const widths={...own.columnWidths};
   for(const column of VIEW_COLUMNS){const value=patch.columnWidths?.[column];if(value!==undefined&&!settings()[`mail.messageListColumnWidth.${column}`]?.locked)widths[column]=Math.max(64,Math.min(800,value));}
   next.columnWidths=widths;
  }else if(!folderViewLocked(field))Object.assign(next,{[field]:patch[field]});
 }
 for(const field of Object.keys(next) as (keyof FolderViewPreferences)[])if(next[field]===undefined)delete next[field];
 setAccountUiPreferences({folderProps:withOverride(all,id,{view:Object.keys(next).length?next:undefined})});
}
export function folderViewFilter(id=activeFolderViewId()):Record<string,unknown>|undefined {
 let filter:Record<string,unknown>|undefined;
 switch(folderViewValue('filter',id)){
  case 'unread':filter={notKeyword:'$seen'};break;case 'read':filter={hasKeyword:'$seen'};break;
  case 'flagged':filter={hasKeyword:'$flagged'};break;case 'attachments':filter={hasAttachment:true};break;
  case 'mentions':filter={text:'mentions:me'};break;
 }
 const query=folderViewValue('filterQuery',id).trim();
 return query?{operator:'AND',conditions:[...(filter?[filter]:[]),{text:query}]}:filter;
}

export function folderViewFontSize(density:'compact'|'cozy'|'relaxed',id=activeFolderViewId()):number {
 const own=id?accountUiPreferences().folderProps?.[id]?.view?.fontSize:undefined;
 return !settings()['mail.messageListFontSize']&&own===undefined?({compact:13,cozy:14,relaxed:15}[density]):folderViewValue('fontSize',id);
}
export function folderViewStyle(density:'compact'|'cozy'|'relaxed',id=activeFolderViewId()):Record<string,string> {
 const size=folderViewFontSize(density,id),delta=size-({compact:13,cozy:14,relaxed:15}[density]),columns=folderViewValue('columns',id);
 return {'--folder-view-columns':columns.map(c=>`${folderColumnWidth(c,id)}px`).join(' '),'--folder-view-width':`${columns.reduce((sum,c)=>sum+folderColumnWidth(c,id)+12,12)}px`,'--folder-view-font':({inherit:'inherit',sans:'sans-serif',serif:'serif',mono:'monospace'}[folderViewValue('font',id)]),'--mail-row-padding':`${({compact:3,cozy:7,relaxed:10}[density])+folderViewValue('rowSpacing',id)}px`,'--mail-row-font':`${size}px`,'--mail-row-line':`${({compact:16,cozy:18,relaxed:21}[density])+Math.ceil(delta*1.3)}px`,'--mail-preview-font':`${Math.max(10,size-2)}px`,'--mail-preview-line':`${({compact:14,cozy:16,relaxed:18}[density])+Math.ceil(delta*1.3)}px`};
}

export function resetFolderColumnWidth(id:string,column:ViewColumn):void {
 if(!folderViewAllowed()||settings()[`mail.messageListColumnWidth.${column}`]?.locked)return;
 const all=accountUiPreferences().folderProps,view=all?.[id]?.view;if(!view)return;
 const widths={...view.columnWidths};delete widths[column];
 setAccountUiPreferences({folderProps:withOverride(all,id,{view:{...view,columnWidths:widths}})});
}
