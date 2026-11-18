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
it('shows quick measurements without automatically scanning cached emails',async()=>{
 render(()=><DatabaseObservability/>);
 expect(await screen.findByText('SQLx · sqlite')).toBeVisible();
 expect(screen.getByText('mailwoman.db')).toBeVisible();
 expect(loadDatabaseReport).toHaveBeenCalledExactlyOnceWith(false);
 expect(screen.queryByRole('columnheader',{name:'Table'})).toBeNull();
});
it('requests detailed inspection explicitly and searches the measured tables',async()=>{
 render(()=><DatabaseObservability/>);await screen.findByText('mailwoman.db');
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

it('makes sustained growth and write origins visible without a payload scan',async()=>{
 const next=fixture();next.growth={status:'rapid-growth',bytesPerMinute:100000000,thresholdBytesPerMinute:67108864,windowMinutes:15,
  samples:[{at:1700000000,dbBytes:100000000,walBytes:0}],writeOrigins:[{origin:'INSERT bodies · cache.rs:1600',statements:30,affectedRows:30,inputBytes:100000000}]};
 vi.mocked(loadDatabaseReport).mockResolvedValue(next);render(()=><DatabaseObservability/>);
 expect(await screen.findByRole('alert')).toHaveTextContent('Sustained rapid growth');
 expect(screen.getByText('INSERT bodies · cache.rs:1600')).toBeVisible();
 expect(loadDatabaseReport).toHaveBeenCalledExactlyOnceWith(false);
});
