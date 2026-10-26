import {adminRequest,type AdminResponse} from '../../screens/Admin/shell/adminApi.ts';
export interface MaintenanceConfig {enabled:boolean;message:string;retryAfterSecs:number}
export interface MaintenanceView extends MaintenanceConfig {configurationInvalid?:boolean}
export interface MaintenanceModeApi {
  read():Promise<AdminResponse<MaintenanceView>>;
  save(value:MaintenanceConfig,etag:string):Promise<AdminResponse<MaintenanceConfig>>;
}
export const maintenanceModeApi:MaintenanceModeApi={
  read:()=>adminRequest('/maintenance/mode'),
  save:(value,ifMatch)=>adminRequest('/maintenance/mode',{method:'PUT',body:{enabled:value.enabled,message:value.message,retryAfterSecs:value.retryAfterSecs},ifMatch}),
};
