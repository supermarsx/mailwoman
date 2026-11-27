import {settings} from '../api/settings.ts';
import {smartArchivePolicy} from './smartArchive.ts';
export const ROW_ACTION_IDS=['view-draft','pin','snooze','label','flag','archive','smart-archive','delete'] as const;
export type RowActionId=typeof ROW_ACTION_IDS[number];
export const ROW_ACTION_LAYOUT_KEY='mail.rowActions';
export const DEFAULT_ROW_ACTION_LAYOUT=ROW_ACTION_IDS.filter(id=>id!=='smart-archive').map(id=>`quick:${id}`);
export function isRowActionLayout(value:unknown):value is string[]{
 if(!Array.isArray(value)||value.length>ROW_ACTION_IDS.length)return false;const seen=new Set<string>();
 return value.every(item=>{if(typeof item!=='string')return false;const [place,id,...extra]=item.split(':');if(extra.length||!['quick','more'].includes(place!)||!ROW_ACTION_IDS.includes(id as RowActionId)||seen.has(id!))return false;seen.add(id!);return true;});
}
export function rowActionLayout(){const entry=settings()[ROW_ACTION_LAYOUT_KEY],raw=entry?.value;const valid=isRowActionLayout(raw),customized=valid&&entry?.source!=='default';let value=[...(valid?raw:DEFAULT_ROW_ACTION_LAYOUT)];const policy=smartArchivePolicy();if(!policy.enabled||policy.replaceArchive)value=value.filter(item=>!item.endsWith(':smart-archive'));else if(!customized&&!value.some(item=>item.endsWith(':smart-archive')))value.splice(value.indexOf('quick:archive')+1,0,'quick:smart-archive');return {quick:value.filter(item=>item.startsWith('quick:')).map(item=>item.slice(6) as RowActionId),more:value.filter(item=>item.startsWith('more:')).map(item=>item.slice(5) as RowActionId),customized};}
export const ROW_ACTION_LABELS:Record<RowActionId,string>={'view-draft':'mail-draft-view',pin:'mail-pin',snooze:'mail-snooze',label:'mail-label',flag:'mail-flag',archive:'mail-archive','smart-archive':'mail-smart-archive',delete:'mail-delete'};
