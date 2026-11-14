import {settings} from '../api/settings.ts';
import {viewerKindFor,type ViewerKind} from '../contracts/viewer.ts';
import tools from './previewTools.json';
export const FILE_PREVIEWS_KEY='reader.filePreviews';
export const PREVIEW_KINDS=['image','pdf','text','html','code','audio','video','message','calendar'] as const;
export type PreviewKind=typeof PREVIEW_KINDS[number];
export type PreviewPlacement='toolbar'|'menu'|'hidden';
export interface FilePreviewConfig{enabled:boolean;convertImages:boolean;maxBytes:number;types:Partial<Record<PreviewKind,boolean>>;imageFormats:string[];textExtensions:string[];tools:Record<string,PreviewPlacement>}
export const PREVIEW_TOOLS=tools;
export const DEFAULT_FILE_PREVIEWS:FilePreviewConfig={enabled:true,convertImages:true,maxBytes:64*1024*1024,types:{},imageFormats:['*'],textExtensions:[],tools:{}};
export function validFilePreviews(value:unknown):value is FilePreviewConfig{
 if(!value||typeof value!=='object'||Array.isArray(value))return false;const row=value as FilePreviewConfig;
 return Object.keys(row).every(key=>['enabled','convertImages','maxBytes','types','imageFormats','textExtensions','tools'].includes(key))&&typeof row.enabled==='boolean'&&typeof row.convertImages==='boolean'&&Number.isInteger(row.maxBytes)&&row.maxBytes>=1024&&row.maxBytes<=256*1024*1024&&!!row.types&&typeof row.types==='object'&&!Array.isArray(row.types)&&Object.entries(row.types).every(([key,value])=>PREVIEW_KINDS.includes(key as PreviewKind)&&typeof value==='boolean')&&[row.imageFormats,row.textExtensions].every(list=>Array.isArray(list)&&list.length<=128&&list.every(item=>typeof item==='string'&&/^(?:\*|[a-z0-9][a-z0-9+_-]{0,39})$/.test(item)))&&!!row.tools&&typeof row.tools==='object'&&!Array.isArray(row.tools)&&Object.entries(row.tools).every(([key,value])=>tools.some(tool=>tool.id===key)&&['toolbar','menu','hidden'].includes(value));
}
export function filePreviews():FilePreviewConfig{const value=settings()[FILE_PREVIEWS_KEY]?.value;return validFilePreviews(value)?value:DEFAULT_FILE_PREVIEWS;}
export function previewToolPlacement(id:string):PreviewPlacement{return filePreviews().tools[id]??'toolbar';}
export function previewToolEnabled(id:string):boolean{return filePreviews().enabled&&previewToolPlacement(id)!=='hidden';}
export function filePreviewAllowed(item:{name:string;mime:string;size:number}):boolean{
 const config=filePreviews(),kind=viewerKindFor(item.mime,item.name);if(!config.enabled||item.size>config.maxBytes||kind==='unsupported'||config.types[kind]===false)return false;
 const extension=item.name.split('.').pop()?.toLowerCase()??'';
 if(kind==='image'&&!config.imageFormats.includes('*')&&!config.imageFormats.includes(extension)&&!config.imageFormats.includes(item.mime.toLowerCase().split('/')[1]?.split(';')[0]??''))return false;
 return !['text','code','html'].includes(kind)||!config.textExtensions.length||config.textExtensions.includes('*')||config.textExtensions.includes(extension);
}
export function previewKindEnabled(kind:ViewerKind):boolean{return kind!=='unsupported'&&filePreviews().enabled&&filePreviews().types[kind]!==false;}
