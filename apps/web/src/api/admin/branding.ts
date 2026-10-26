import { adminRequest } from '../../screens/Admin/shell/adminApi.ts';
export interface BrandImages {logoId:string|null;iconMasterId:string|null;loginBackgroundId:string|null}
export interface Branding {brandName:string;loginNotice:string|null;images:BrandImages;assets:{logoUrl:string|null;faviconUrl:string|null;loginBackgroundUrl:string|null}}
export interface BrandRead {data:Branding;etag:string}
export type BrandKind='logo'|'icon_master'|'background';
export interface BrandingApi {read:()=>Promise<BrandRead>;save:(value:Branding,etag:string)=>Promise<BrandRead>;upload:(kind:BrandKind,file:File)=>Promise<string>}
export const brandingApi:BrandingApi={
 async read(){const r=await adminRequest<Branding>('/branding');return {data:r.data,etag:r.etag??''};},
 async save(value,etag){const {brandName,loginNotice,images}=value;const r=await adminRequest<Branding>('/branding',{method:'PUT',ifMatch:etag,body:{brandName,loginNotice,images}});return {data:r.data,etag:r.etag??''};},
 async upload(kind,file){if(file.size>1_000_000)throw new Error('Image must be at most 1 MB');const bytes=new Uint8Array(await file.arrayBuffer());let raw='';for(let p=0;p<bytes.length;p+=8192)raw+=String.fromCharCode(...bytes.subarray(p,p+8192));return(await adminRequest<{id:string}>('/branding/assets',{method:'POST',body:{kind,bytesBase64:btoa(raw)}})).data.id;},
};
