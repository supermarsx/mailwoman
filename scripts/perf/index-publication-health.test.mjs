import test from 'node:test';
import assert from 'node:assert/strict';
import {indexPublicationReport,qualifiesIndexPublication} from './index-publication-health.mjs';
export const healthyPublication = {running:true,poisoned:false,requiresFullRebuild:false,pendingJobs:0,pendingBytes:0,queuedJobs:0,submittedJobs:20,completedJobs:20,failedJobs:0,publishedBatches:8,publishedChanges:20,revalidations:0,pendingRepairMessages:0,erasingAccounts:0,maxPendingJobs:128,maxPendingBytes:16777216,maxBatchDocuments:4096};
const proof=(after={})=>({before:{searchIndex:{publication:healthyPublication}},after:{searchIndex:{publication:{...healthyPublication,submittedJobs:40,completedJobs:40,...after}}}});
test('publication health strips private fields and rejects malformed or oversized samples',()=>{
 assert.equal(indexPublicationReport(undefined),null);
 assert.deepEqual(indexPublicationReport({...healthyPublication,accountId:'private',path:'/private',repairIds:['private']}),healthyPublication);
 for(const patch of [{submittedJobs:NaN},{completedJobs:-1},{pendingBytes:16777217},{maxPendingJobs:1025},{maxBatchDocuments:4097},{poisoned:0},{running:'true'},{pendingJobs:129}])assert.throws(()=>indexPublicationReport({...healthyPublication,...patch}));
});
test('capacity proof requires a quiescent healthy owner and completed work at both boundaries',()=>{
 assert.equal(qualifiesIndexPublication({}),false);
 assert.equal(qualifiesIndexPublication(proof()),true);
 for(const patch of [{running:false},{poisoned:true},{requiresFullRebuild:true},{pendingJobs:1},{pendingBytes:1},{queuedJobs:1},{pendingRepairMessages:1},{erasingAccounts:1},{failedJobs:1},{completedJobs:39},{completedJobs:41},{submittedJobs:19},{publishedBatches:0},{revalidations:-1}])assert.equal(qualifiesIndexPublication(proof(patch)),false,JSON.stringify(patch));
 const dirtyBefore=proof();dirtyBefore.before.searchIndex.publication={...healthyPublication,pendingRepairMessages:1};
 assert.equal(qualifiesIndexPublication(dirtyBefore),false);
});
