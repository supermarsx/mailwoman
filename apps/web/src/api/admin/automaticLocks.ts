import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';
export interface AutomaticLock { surface:'admin'|'webmail'; ip:string; failures:number; lockedUntil:number; revision:string }
export interface AutomaticLocksApi { list():Promise<AutomaticLock[]>; unlock(lock:AutomaticLock):Promise<void> }
export const automaticLocksApi:AutomaticLocksApi={
 async list(){return (await adminRequest<{locks:AutomaticLock[]}>('/bans/automatic')).data.locks;},
 async unlock(lock){await adminRequest(`/bans/automatic/${lock.surface}/${encodeURIComponent(lock.ip)}`,{method:'DELETE',ifMatch:`"${lock.revision}"`});},
};
