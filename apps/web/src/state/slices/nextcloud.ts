/** Account-owned link availability. This reads local metadata only; opening the
 * composer never probes an upstream server or replays stored credentials. */
import {createSignal,getOwner,onCleanup,type Accessor} from 'solid-js';
import {NextcloudService,type Fetcher} from '../../modules/nextcloud/service.ts';
import {apiFetch} from '../../api/http.ts';
import {withBase} from '../../api/basePath.ts';
import {observePreferenceAccount,preferenceAccount,preferenceAccountIsCurrent} from '../../api/accountPreferenceLifecycle.ts';
export interface NextcloudSlice {
  readonly service:NextcloudService;
  enabled:Accessor<boolean>;
  setEnabled(on:boolean):void;
  ensureEnabled():Promise<void>;
}
export function createNextcloudSlice(fetcher?:Fetcher):NextcloudSlice {
  const service=new NextcloudService(fetcher);
  const [enabled,setEnabled]=createSignal(false);
  let generation=0;let pending:Promise<void>|undefined;
  const stop=observePreferenceAccount(()=>{generation++;pending=undefined;setEnabled(false);});
  if(getOwner())onCleanup(stop);
  function ensureEnabled():Promise<void>{
    const account=preferenceAccount();if(!account)return Promise.resolve();
    if(pending)return pending;
    const epoch=generation;
    const request=fetcher??((path:string)=>apiFetch(path,{credentials:'same-origin'}));
    const work=(async()=>{try{
      const response=await request(withBase('/api/account/nextcloud'));
      if(!response.ok)throw new Error('Unavailable');
      const body=await response.json() as {accountId:string;links:{ready:boolean}[]};
      if(epoch===generation&&preferenceAccountIsCurrent(account))setEnabled(body.accountId===account.id&&body.links.some(link=>link.ready));
    }catch{if(epoch===generation)setEnabled(false);}finally{if(epoch===generation)pending=undefined;}})();
    pending=work;return work;
  }
  return {service,enabled,setEnabled,ensureEnabled};
}
