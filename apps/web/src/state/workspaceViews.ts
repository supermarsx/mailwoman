import {settings} from '../api/settings.ts';
import type {IconName} from '../components/Icon.tsx';
export const WORKSPACE_VIEWS=[
 {id:'notifications',label:'common-module-notifications',icon:'bell'},
 {id:'calendar',label:'common-module-calendar',icon:'calendar'},
 {id:'tasks',label:'common-module-tasks',icon:'checklist'},
 {id:'notes',label:'common-module-notes',icon:'book'},
 {id:'contacts',label:'common-module-contacts',icon:'person'},
 {id:'keys',label:'common-module-keys',icon:'key'},
 {id:'attachments',label:'common-attach-title',icon:'attachment'},
 {id:'outbox',label:'mail-nav-outbox',icon:'clock'},
 {id:'templates',label:'response-templates-workspace',icon:'book'},
] as const satisfies readonly {id:string;label:string;icon:IconName}[];
export type WorkspaceViewId=typeof WORKSPACE_VIEWS[number]['id'];
export function viewMode(id:string):'visible'|'hidden'|'notable'{const value=settings()[`views.${id}.visibility`]?.value;return value==='hidden'||value==='notable'?value:'visible';}
export function viewShown(id:string,count:number|null,active=false):boolean{const mode=viewMode(id);return mode==='visible'||mode==='notable'&&(active||count===null||count>0);}
export function viewName(id:string,fallback:string):string {const value=settings()[`views.${id}.name`]?.value;return typeof value==='string'&&value.trim()?value.trim():fallback;}
export const viewCountsShown=(id:string)=>settings()[`views.${id}.showCount`]?.value!==false;
