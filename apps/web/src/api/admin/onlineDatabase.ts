import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';
export interface OnlineCopyProgress {
 tableIndex:number;tableName:string|null;tableCount:number;copiedRows:number;copiedBytes:number;replayedKeys:number;pending:number;seedComplete:boolean;valid:boolean;
}
export interface OnlineConversionReport {
 id:string;phase:string;startedAt:number;updatedAt:number;progress:OnlineCopyProgress|null;error:string|null;retainedPrevious:boolean;
}
export const loadOnlineConversion=async(signal?:AbortSignal):Promise<OnlineConversionReport|null>=>(await adminRequest<OnlineConversionReport|null>('/maintenance/database-online',signal?{signal}:{})).data;
export const startOnlineConversion=async():Promise<OnlineConversionReport>=>(await adminRequest<OnlineConversionReport>('/maintenance/database-online',{method:'POST',body:{}})).data;
export const cancelOnlineConversion=async():Promise<boolean>=>(await adminRequest<{cancelRequested:boolean}>('/maintenance/database-online',{method:'DELETE'})).data.cancelRequested;
