// Copies test harness files only into an explicit, separate Git worktree.
// Baseline production sources and manifests are never modified.
import {readFile,writeFile,copyFile,mkdir,stat,realpath} from 'node:fs/promises';
import {resolve,dirname,sep} from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';

const source=await realpath(resolve(dirname(fileURLToPath(import.meta.url)),'../..'));
const target=await realpath(resolve(process.argv[2]??''));
if(!process.argv[2]||target===source||!(await stat(resolve(target,'.git'))).isFile())throw new Error('Explicit separate Git worktree required');
const files=['scripts/perf/http-load.mjs','scripts/perf/http-campaign.mjs','scripts/perf/http-real-fixture.mjs','scripts/perf/fixture-clients.mjs','scripts/perf/fixture-requests.mjs','scripts/perf/representative-workload.mjs','scripts/perf/http-external-driver.mjs','scripts/perf/external-control.mjs','crates/mw-server/tests/request_load_fixture.rs','crates/mw-server/tests/common/request_load_control.rs'];
const hashes={};
for(const file of files){
 const destination=resolve(target,file);if(!destination.startsWith(target+sep))throw new Error('Invalid overlay path');
 await mkdir(dirname(destination),{recursive:true});await copyFile(resolve(source,file),destination);
 hashes[file]=createHash('sha256').update(await readFile(destination)).digest('hex');
}
const adapter=resolve(target,'crates/mw-server/tests/common/request_load_diagnostics.rs');
await mkdir(dirname(adapter),{recursive:true});
await writeFile(adapter,`//! Baseline fixture adapter: no new production instrumentation required.
pub async fn snapshot(store:&mw_store::Store,directory:&std::path::Path)->serde_json::Value {
 let pool=store.store_diagnostics().await.ok().map(|v|serde_json::json!({"backend":v.backend,"open":v.pool_size,"idle":v.pool_idle,"dbPingMs":v.db_ping_ms}));
 serde_json::json!({"pool":pool,"resources":mw_observability::resource_snapshot::snapshot(directory),"requestProfile":null,"profileEnabled":false,"adapter":"baseline-compatible"})
}
pub fn reset(){}
`);
console.log(JSON.stringify({overlay:'test harness only; production backend unchanged',files:hashes,adapter:'baseline-compatible'},null,2));
