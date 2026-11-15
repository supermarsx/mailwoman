import {ApiError,NetworkError} from './errors.ts';
/** Worker transport cannot read document.cookie. The caller acquires its CSRF
 * token from live /api/me, and supplies verified account-binding headers. Every
 * URL remains inside this Mailwoman origin and deployment prefix. */
export async function workerHttp(base:string,path:string,init:RequestInit={}):Promise<Response>{
 const root=new URL(base,globalThis.location?.href),target=new URL(path,root.origin);
 const prefix=root.pathname.replace(/\/$/,'');
 if(root.origin!==globalThis.location?.origin||target.origin!==root.origin||target.username||target.password||!target.pathname.startsWith(`${prefix}/api/`)&&!target.pathname.startsWith(`${prefix}/jmap/`))throw new ApiError(400,'Worker request outside Mailwoman');
 let response:Response;
 try{response=await fetch(target.href,{...init,credentials:'same-origin',redirect:'error',cache:'no-store'});}catch{throw new NetworkError('Worker connection unavailable');}
 if(!response.ok){const body=await response.clone().json().catch(()=>({})) as {error?:string;code?:string};throw new ApiError(response.status,body.error??'Worker request refused',{...(body.code?{code:body.code}:{}),...(/^\d+$/.test(response.headers.get('retry-after')??'')?{retryAfterSecs:Number(response.headers.get('retry-after'))}:{})});}
 return response;
}
