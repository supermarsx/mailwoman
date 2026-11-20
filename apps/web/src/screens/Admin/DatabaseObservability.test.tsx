import {beforeEach,expect,it,vi} from 'vitest';
import {fireEvent,render,screen,waitFor,within} from '@solidjs/testing-library';
import {DatabaseObservability} from './DatabaseObservability.tsx';
import {loadDatabaseReport,type DatabaseReport} from '../../api/admin/database.ts';
vi.mock('../../api/admin/database.ts',()=>({loadDatabaseReport:vi.fn()}));
const fixture=():DatabaseReport=>({observedAt:'2026-11-18T12:00:00Z',driver:'SQLx',
 database:{backend:'sqlite',serverVersion:'3.49.0',name:'mailwoman.db',host:null,port:null,journalMode:'wal'},
 diagnostics:{dbBytes:104854867968,walBytes:0,dbPingMs:1,migrationApplied:45,migrationExpected:45},
 pool:{open:2,idle:1,busy:1,minimum:1,maximum:4,acquireTimeoutMs:30000,idleTimeoutMs:600000,maxLifetimeMs:1800000,closed:false},
 storage:{pageSize:4096,pageCount:25600000,freePages:0,freeBytes:0,occupiedBytes:104857600000},
 migrations:{applied:45,expected:45,pending:[],failed:[],unknown:[]},details:null});
beforeEach(()=>{vi.mocked(loadDatabaseReport).mockReset();vi.mocked(loadDatabaseReport).mockResolvedValue(fixture());});
it('loads the storage breakdown after quick measurements',async()=>{
 render(()=><DatabaseObservability/>);
 expect(await screen.findByText('SQLx · sqlite')).toBeVisible();
 expect(screen.getByText('mailwoman.db')).toBeVisible();
 await waitFor(()=>expect(loadDatabaseReport).toHaveBeenCalledTimes(2));expect(loadDatabaseReport).toHaveBeenNthCalledWith(1,false);expect(loadDatabaseReport).toHaveBeenNthCalledWith(2,true);
 expect(screen.queryByRole('columnheader',{name:'Table'})).toBeNull();
});
it('requests detailed inspection explicitly and searches the measured tables',async()=>{
 render(()=><DatabaseObservability/>);await screen.findByText('mailwoman.db');await waitFor(()=>expect(screen.getByRole('button',{name:'Inspect storage'})).not.toBeDisabled());
 const next=fixture();next.details={observedAt:next.observedAt,limited:false,notes:[],
  bodies:{count:20,bytes:1000000,orphanCount:19,orphanBytes:950000},
  tables:[{name:'bodies',bytes:1000000,rows:20,rowsEstimated:false},{name:'sessions',bytes:4096,rows:1,rowsEstimated:false}]};
 vi.mocked(loadDatabaseReport).mockResolvedValue(next);
 fireEvent.click(screen.getByRole('button',{name:'Inspect storage'}));
 await waitFor(()=>expect(loadDatabaseReport).toHaveBeenLastCalledWith(true));
 expect(await screen.findByText('Unreferenced bodies')).toBeVisible();
 const table=screen.getByRole('columnheader',{name:'Table'}).closest('table')!;
 expect(within(table).getByText('sessions')).toBeVisible();
 fireEvent.input(screen.getByRole('searchbox'),{target:{value:'bodies'}});
 expect(within(table).queryByText('sessions')).toBeNull();
 expect(within(table).getByText('bodies')).toBeVisible();
});

it('labels sampled byte lower bounds and never subtracts them into referenced totals',async()=>{
 const next=fixture();next.details={observedAt:next.observedAt,limited:true,notes:['body-totals-unavailable'],
 bodies:{count:800,bytes:524288,bytesLowerBound:true,orphanCount:800,orphanBytes:819200},
 tables:[{name:'bodies',bytes:null,payloadBytes:524288,payloadBytesLowerBound:true,payloadSampleRows:512,rows:800,rowsEstimated:false,columns:[{name:'sealed_bytes',bytes:524288,bytesLowerBound:true}]}]};
 vi.mocked(loadDatabaseReport).mockResolvedValue(next);render(()=><DatabaseObservability/>);
 expect(await screen.findByText(/not estimated totals/)).toBeVisible();
 const table=screen.getByRole('columnheader',{name:'Table'}).closest('table')!;
 const row=within(table).getByText('bodies').closest('tr')!;
 expect(row).toHaveTextContent('≥');expect(row).toHaveTextContent('Physical scan limited');expect(row).toHaveTextContent('800');
 expect(screen.getByText(/including inline images and attachments/)).toBeVisible();
 const referenced=screen.getByText('Referenced cached body bytes').closest('tr')!;
 expect(referenced).toHaveTextContent('Not available');
});

it('makes sustained growth and write origins visible without a payload scan',async()=>{
 const next=fixture();next.growth={status:'rapid-growth',bytesPerMinute:100000000,thresholdBytesPerMinute:67108864,windowMinutes:15,
  samples:[{at:1700000000,dbBytes:100000000,walBytes:0}],writeOrigins:[{origin:'INSERT bodies · cache.rs:1600',statements:30,affectedRows:30,inputBytes:100000000}]};
 vi.mocked(loadDatabaseReport).mockResolvedValue(next);render(()=><DatabaseObservability/>);
 expect(await screen.findByRole('alert')).toHaveTextContent('Sustained rapid growth');
 expect(screen.getByText('INSERT bodies · cache.rs:1600')).toBeVisible();
 await waitFor(()=>expect(loadDatabaseReport).toHaveBeenCalledTimes(2));expect(loadDatabaseReport).toHaveBeenNthCalledWith(1,false);expect(loadDatabaseReport).toHaveBeenNthCalledWith(2,true);
});

it('explains demand-managed pools, restart boundaries and process acquisition counters',async()=>{
 const report=fixture();report.pool.scaling='sqlx-demand-managed';report.pool.ceilingChangesRequireRestart=true;report.pool.dynamicCeilingResize=false;
 const operation={calls:20,failures:0,totalMicros:2000,maxMicros:100};report.pool.operations={read:operation,write:{...operation,calls:2},other:operation,acquire:{calls:42,failures:1,totalMicros:12000,maxMicros:5000}};
 vi.mocked(loadDatabaseReport).mockResolvedValue(report);render(()=><DatabaseObservability/>);await screen.findByText('mailwoman.db');
 expect(screen.getByText('Pool acquisitions').closest('tr')).toHaveTextContent('42');expect(screen.getByText('Acquisition failures').closest('tr')).toHaveTextContent('1');
 expect(screen.getByText(/Connection limits and timings change only after a full server restart/)).toBeVisible();
 expect(screen.getByText(/Counters cover this server process/)).toBeVisible();
});

it('shows signed net removal separately from file allocation and measured cleanup receipts',async()=>{
 const report=fixture();report.growth={status:'normal',basis:'occupied-pages',bytesPerMinute:-4096,thresholdBytesPerMinute:67108864,windowMinutes:15,
  samples:[{at:1000,dbBytes:16384,walBytes:0,occupiedBytes:12288,freeBytes:4096,segment:1},{at:1060,dbBytes:16384,walBytes:2048,occupiedBytes:8192,freeBytes:8192,segment:1}],
  changes:{from:1000,to:1060,seconds:60,occupiedBytes:-4096,dbBytes:0,walBytes:2048,freeBytes:4096,knownCleanup:null},
  knownCleanupTotal:{removedBodies:2,removedBodyBytes:12345,cleanedEnvelopes:1,removedEnvelopeBytes:200,removedChangeRows:17,reclaimedPages:0,retiredFileBytes:1000000},
  writeOperations:[{operation:'DELETE',statements:3,affectedRows:19,inputBytes:24}],writeOrigins:[]};
 vi.mocked(loadDatabaseReport).mockResolvedValue(report);render(()=><DatabaseObservability/>);
 const occupied=await screen.findByText('Net occupied-page change');expect(occupied.closest('tr')).toHaveTextContent('−');
 expect(screen.getByText('Active file allocation change').closest('tr')).not.toHaveTextContent('−');
 expect(screen.getByText('Reusable space change').closest('tr')).toHaveTextContent('+');
 const cleanup=screen.getByRole('region',{name:'Known completed cleanup (this process)'});
 expect(within(cleanup).getByText('Sync-history rows removed').closest('tr')).toHaveTextContent('17');
 expect(within(cleanup).getByText('Retained database files removed')).toBeVisible();
 expect(within(cleanup).getByText(/Removed payload does not mean file shrink/)).toBeVisible();
 const operations=screen.getByRole('table',{name:'Statement activity by operation'});
 expect(within(operations).getByText('DELETE').closest('tr')).toHaveTextContent('19');
 expect(screen.queryByRole('alert')).toBeNull();
});

it('keeps legacy unknown cleanup and allocation-only measurements honest',async()=>{
 const report=fixture();report.growth={status:'warming-up',basis:'database-allocation',bytesPerMinute:null,thresholdBytesPerMinute:1,windowMinutes:15,samples:[],writeOrigins:[]};
 vi.mocked(loadDatabaseReport).mockResolvedValue(report);render(()=><DatabaseObservability/>);
 expect((await screen.findByText('Net database allocation rate')).closest('tr')).toHaveTextContent('Not available');
 expect(screen.queryByText('Known completed cleanup (this process)')).toBeNull();
 expect(screen.getByText(/Net change includes deletions and space reuse/)).toBeVisible();
});
