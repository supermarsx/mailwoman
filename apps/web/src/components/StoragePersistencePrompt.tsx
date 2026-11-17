import {createEffect,createSignal,onCleanup,type JSX} from 'solid-js';
import {settings} from '../api/settings.ts';
import {preferenceAccount,preferenceAccountIsCurrent,observePreferenceAccount} from '../api/accountPreferenceLifecycle.ts';
import {browserStorageStatus,requestBrowserPersistence} from '../offline/storagePermission.ts';
import {toast} from './toast/toast.ts';
import {t,loadCatalog} from '../i18n/index.ts';
export function StoragePersistencePrompt():JSX.Element{
 const [account,setAccount]=createSignal(preferenceAccount());onCleanup(observePreferenceAccount(setAccount));
 let generation=0;const shown=new Set<string>();
 createEffect(()=>{
  const owner=account(),map=settings();const epoch=++generation;
  if(!owner||map['offline.enabled']?.value!==true||map['offline.persistentStorageAllowed']?.value!==true||map['offline.requestPersistence']?.value===false){toast.dismiss('storage-persistence');return;}
  const key=`mw.storage-offer:${owner.storageAuthority??owner.id}`;if(shown.has(key))return;
  try{if(Date.now()-Number(localStorage.getItem(key)??0)<30*86400000)return;}catch{/* Memory-only suppression is sufficient. */}
  void Promise.all([browserStorageStatus(),loadCatalog('settings')]).then(([status])=>{
   if(epoch!==generation||!preferenceAccountIsCurrent(owner)||!status.supported||status.persistent)return;
   shown.add(key);
   toast.info(t('settings-storage-offer'),{id:'storage-persistence',persistent:true,action:{label:t('settings-storage-request'),run:async()=>{if(preferenceAccountIsCurrent(owner)&&settings()['offline.persistentStorageAllowed']?.value===true)await requestBrowserPersistence();}},onClose:()=>{try{localStorage.setItem(key,String(Date.now()));}catch{/* Browser may refuse local storage. */}}});
  }).catch(()=>undefined);
 });
 onCleanup(()=>{generation++;toast.dismiss('storage-persistence');});return <></>;
}
