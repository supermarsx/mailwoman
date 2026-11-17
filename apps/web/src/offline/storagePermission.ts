import {formatBytes} from '../i18n/format.ts';
import {toast} from '../components/toast/toast.ts';
import {t,loadCatalog} from '../i18n/index.ts';
export interface StorageStatus {supported:boolean;persistent:boolean;usage?:number|undefined;quota?:number|undefined}
export async function browserStorageStatus():Promise<StorageStatus>{
 const storage=globalThis.navigator?.storage;if(!storage||typeof storage.persist!=='function')return {supported:false,persistent:false};
 const [persistent,estimate]=await Promise.all([storage.persisted(),storage.estimate()]);
 return {supported:true,persistent,usage:estimate.usage,quota:estimate.quota};
}
export async function requestBrowserPersistence():Promise<boolean>{
 // Call directly from a user gesture; browsers decide whether a dialog is needed.
 const request=globalThis.navigator?.storage?.persist?.();if(!request)return false;
 try{const granted=await request;await loadCatalog('settings');toast.show(granted?'success':'warning',t(granted?'settings-storage-granted':'settings-storage-refused'));return granted;}
 catch{await loadCatalog('settings');toast.error(t('settings-permissions-error'));return false;}
}
export const storageSize=(bytes:number|undefined):string=>bytes===undefined?'—':formatBytes(bytes);
