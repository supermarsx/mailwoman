/** Preserve per-catalog lazy loading while changing the production wire format. */
// eslint-disable-next-line mw/no-global-fetch -- Public hashed text assets; no account API requests or writes.
export function catalogAssetLoaders(urls:Record<string,string>,request:typeof fetch=(...args)=>fetch(...args)):Record<string,()=>Promise<string>>{
 return Object.fromEntries(Object.entries(urls).map(([key,url])=>[key,async()=>{
  const response=await request(url);
  if(!response.ok)throw new Error(`Translation catalog unavailable (${response.status})`);
  return response.text();
 }]));
}
