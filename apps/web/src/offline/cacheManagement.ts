import {preferenceAccount,preferenceAccountIsCurrent} from '../api/accountPreferenceLifecycle.ts';
import {offlineNamespace} from './runtime.ts';
import {offlineVault} from './vault.ts';
import {offlineHttpNamespace,offlineServerBase} from './cacheScope.ts';

export async function offlineCacheUsage():Promise<{bytes:number;entries:number;protectedBytes:number}|undefined>{
 const owner=preferenceAccount();if(!owner?.storageAuthority)return;
 const vault=await offlineVault();
 const usage=await vault.usage(JSON.stringify(['offline-device',offlineServerBase()]));
 return preferenceAccountIsCurrent(owner)?usage:undefined;
}
/** Read caches only: unsent drafts and pending outbound work survive a purge. */
export async function purgeOfflineReadCache():Promise<void>{
 const owner=preferenceAccount();if(!owner?.storageAuthority)return;
 const vault=await offlineVault();if(!preferenceAccountIsCurrent(owner))return;
 await vault.clear(offlineNamespace(offlineServerBase(),{accountId:owner.id,storageAuthority:owner.storageAuthority}));
 await vault.clear(offlineHttpNamespace(owner));
}
