import {basePath} from '../api/basePath.ts';
/** Kept independent of the transport/platform import cycle. */
export function offlineServerBase():string{
 const config=(globalThis as {__MW_CONFIG__?:{native?:boolean;serverUrl?:string}}).__MW_CONFIG__;
 if(config?.native&&config.serverUrl)return `native:${config.serverUrl.replace(/\/$/,'')}`;
 return `${globalThis.location?.origin??''}${basePath()}`;
}
export function offlineHttpNamespace(owner:{id:string;storageAuthority?:string}):string{return JSON.stringify(['offline-http',offlineServerBase(),owner.storageAuthority,owner.id]);}
