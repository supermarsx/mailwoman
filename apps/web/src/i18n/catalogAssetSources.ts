// The build plugin fills the key list without importing every asset URL into the
// initial application bundle. Fetchers and URLs arrive with the first catalog.
const keys:string[]=/* @mw-catalog-keys */ [];
export default Object.fromEntries(keys.map(key=>[key,async()=>{
 const {default:loaders}=await import('./catalogAssetUrls.ts');
 const loader=loaders[key];
 if(!loader)throw new Error(`Missing translation catalog: ${key}`);
 return loader();
}]));
