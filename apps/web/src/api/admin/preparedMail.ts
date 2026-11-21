import {adminRequest} from '../../screens/Admin/shell/adminApi.ts';
/** Fixed scalar aggregate only; never principal, account, ID or message data. */
export interface PreparedMailStatus {
 entries:number|null;accounts:number|null;pendingBuilds:number|null;invalidatedEntries:number|null;
 chargedBytes:number;admittedJobs:number;activePreparers:number;budgetBytes:number|null;maxEntries:number|null;maxAccounts:number|null;enabled:boolean|null;idleSeconds:number|null;
 eligibleCandidates:number;readyCandidateHits:number;candidateMisses:number;completedBuilds:number;evictions:number;seedCalls:number;seededRows:number;rawResults:number;compatibilityValueResults:number;
 handledInvalidationHints:number;invalidationErrors:number;invalidationPollCalls:number;invalidationAckCalls:number;
}
export async function loadPreparedMailStatus(signal?:AbortSignal):Promise<PreparedMailStatus>{return (await adminRequest<PreparedMailStatus>('/maintenance/prepared-mail',signal?{signal}:{})).data;}
