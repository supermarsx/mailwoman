import {apiFetch} from './http.ts';
export interface PackageCheckpoint {time:string;location:string;message:string}
export interface TrackedPackage {id:string;label:string;trackingNumber:string;carrier:string;status:string;estimatedDelivery:string|null;checkpoints:PackageCheckpoint[];attemptedAt:number|null;updatedAt:number|null}
export interface PackagesView {accountId:string;revision:string|null;available:boolean;refreshSecs:number;state:{enabled:boolean;packages:TrackedPackage[]}}
export interface PackageDraft {label:string;trackingNumber:string;carrier:string}
export class PackageError extends Error {constructor(readonly code:string){super(code);}}
export function packagesApi(accountId:string,fetcher:typeof apiFetch=apiFetch) {
  async function request(path:string,method='GET',body?:unknown):Promise<unknown> {
    const response=await fetcher(`/api/account/packages${path}`,{method,credentials:'same-origin',...(body?{headers:{'Content-Type':'application/json'},body:JSON.stringify(body)}:{})});
    const data:unknown=await response.json();
    if(!response.ok)throw new PackageError(data&&typeof data==='object'&&'code' in data&&typeof data.code==='string'?data.code:'packageUnavailable');
    return data;
  }
  return {
    async load():Promise<PackagesView>{
      const data=await request('') as PackagesView;
      if(data.accountId!==accountId)throw new PackageError('packageChanged');
      if(!data.state||!Array.isArray(data.state.packages)||data.state.packages.length>100)throw new PackageError('packageUnavailable');
      return data;
    },
    async preference(enabled:boolean,revision:string|null):Promise<void>{await request('','PUT',{accountId,revision,enabled});},
    async save(draft:PackageDraft,revision:string|null,id?:string):Promise<void>{await request(id?`/${encodeURIComponent(id)}`:'',id?'PUT':'POST',{accountId,revision,...draft});},
    async action(id:string,revision:string,action:'refresh'|'remove'):Promise<void>{await request(`/${encodeURIComponent(id)}${action==='refresh'?'/refresh':''}`,action==='refresh'?'POST':'DELETE',{accountId,revision});},
  };
}
export type PackagesApi=ReturnType<typeof packagesApi>;
