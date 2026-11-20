// Aggregate-only proof that successful HTTP throughput is not borrowed from
// uncompleted or failed index publication. Missing legacy evidence is unreported.
const counters = ['pendingJobs','pendingBytes','queuedJobs','submittedJobs','completedJobs','failedJobs','publishedBatches','publishedChanges','revalidations','pendingRepairMessages','erasingAccounts','maxPendingJobs','maxPendingBytes','maxBatchDocuments'];
const booleans = ['running','poisoned','requiresFullRebuild'];
export function indexPublicationReport(value) {
 if(value===undefined)return null;
 if(!value||counters.some(key=>!Number.isSafeInteger(value[key])||value[key]<0)||booleans.some(key=>typeof value[key]!=='boolean')||value.maxPendingJobs<1||value.maxPendingJobs>1024||value.maxPendingBytes<4096||value.maxPendingBytes>128*1024*1024||value.maxBatchDocuments<1||value.maxBatchDocuments>4096||value.pendingJobs>value.maxPendingJobs||value.pendingBytes>value.maxPendingBytes||value.queuedJobs>value.maxPendingJobs)throw new Error('Invalid bounded index publication health');
 return Object.fromEntries([...counters,...booleans].map(key=>[key,value[key]]));
}
export function qualifiesIndexPublication(diagnostics) {
 let before,after;
 try {before=indexPublicationReport(diagnostics?.before?.searchIndex?.publication);after=indexPublicationReport(diagnostics?.after?.searchIndex?.publication);}catch{return false;}
 if(!before||!after)return false;
 if(![before,after].every(value=>value.running&&!value.poisoned&&!value.requiresFullRebuild&&value.pendingJobs===0&&value.pendingBytes===0&&value.queuedJobs===0&&value.pendingRepairMessages===0&&value.erasingAccounts===0))return false;
 if(['submittedJobs','completedJobs','failedJobs','publishedBatches','publishedChanges','revalidations'].some(key=>after[key]<before[key]))return false;
 return after.failedJobs===before.failedJobs&&after.submittedJobs-before.submittedJobs===after.completedJobs-before.completedJobs;
}
