import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';
export interface OnlineCopyProgress {
 tableIndex:number;tableName:string|null;tableCount:number;copiedRows:number;copiedBytes:number;replayedKeys:number;pending:number;seedComplete:boolean;valid:boolean;
}
export interface OnlineConversionReport {
 id:string;phase:string;startedAt:number;updatedAt:number;progress:OnlineCopyProgress|null;error:string|null;retainedPrevious:boolean;
 /** Unix seconds; absent on reports written by older servers. */
 finishedAt?:number|null;
 /** Server-measured elapsed time, frozen once the conversion stops. */
 elapsedMs?:number|null;
 /** Phase elapsed times used as denominators for the average rates. */
 copyElapsedMs?:number|null;replayElapsedMs?:number|null;
 averageCopyBytesPerSecond?:number|null;
 averageReplayKeysPerSecond?:number|null;
}
export const loadOnlineConversion=async(signal?:AbortSignal):Promise<OnlineConversionReport|null>=>(await adminRequest<OnlineConversionReport|null>('/maintenance/database-online',signal?{signal}:{})).data;
export const startOnlineConversion=async():Promise<OnlineConversionReport>=>(await adminRequest<OnlineConversionReport>('/maintenance/database-online',{method:'POST',body:{}})).data;
export const cancelOnlineConversion=async():Promise<boolean>=>(await adminRequest<{cancelRequested:boolean}>('/maintenance/database-online',{method:'DELETE'})).data.cancelRequested;
