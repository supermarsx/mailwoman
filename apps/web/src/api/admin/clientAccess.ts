import {adminRequest,type AdminResponse} from '../../screens/Admin/shell/adminApi.ts';
export interface ClientAccessView {pskEnabled:boolean;slugEnabled:boolean;slug:string|null;pskConfigured:boolean;grantHours:number;clientEntryPath:string}
export interface ClientAccessChange {pskEnabled:boolean;slugEnabled:boolean;slug:string|null;grantHours:number;psk?:string;clearPsk?:boolean}
export interface ClientAccessApi {read():Promise<AdminResponse<ClientAccessView>>;save(change:ClientAccessChange,etag:string):Promise<AdminResponse<ClientAccessView>>}
export const clientAccessApi:ClientAccessApi={read:()=>adminRequest('/client-access'),save:(change,ifMatch)=>adminRequest('/client-access',{method:'PUT',body:change,ifMatch})};
export const validClientSlug=(slug:string):boolean=>/^[a-z0-9](?:[a-z0-9-]{2,62})[a-z0-9]$/.test(slug);
export function validClientPsk(psk:string):boolean{const length=new TextEncoder().encode(psk).length;return psk.trim().length>0&&length>=16&&length<=256;}
