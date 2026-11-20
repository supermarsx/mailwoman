import {beforeEach,expect,it,vi} from 'vitest';
import {Show} from 'solid-js';
import {fireEvent,screen,waitFor} from '@solidjs/testing-library';
import {CacheMaintenance} from './CacheMaintenance.tsx';
import {renderInShell,holding} from './shell/testkit.tsx';
import {loadDatabaseMaintenance} from '../../api/admin/databaseMaintenance.ts';
import {listAdminSettings,type AdminSetting} from '../../api/admin/settings.ts';
import {adminRequest} from './shell/adminApi.ts';
import {useShell} from './shell/context.ts';
vi.mock('../../api/admin/databaseMaintenance.ts',()=>({loadDatabaseMaintenance:vi.fn()}));
vi.mock('../../api/admin/settings.ts',()=>({listAdminSettings:vi.fn()}));
vi.mock('./shell/adminApi.ts',async original=>({...await original<typeof import('./shell/adminApi.ts')>(),adminRequest:vi.fn()}));
vi.mock('./Settings/InlineSettingEditor.tsx',()=>({InlineSettingEditor:(props:{settingKey:string;label?:string;setting:AdminSetting})=><div data-testid={props.settingKey}>{props.label??props.settingKey}: {String(props.setting.effective.value)}</div>}));
const report={backend:'sqlite' as const,running:false,lastReport:null};
beforeEach(()=>{
 vi.mocked(loadDatabaseMaintenance).mockReset().mockResolvedValue(report);
 vi.mocked(adminRequest).mockReset().mockResolvedValue({data:{},status:200,etag:null});
 vi.mocked(listAdminSettings).mockReset().mockResolvedValue({kind:'ok',etag:null,data:[
  {key:'maintenance.database.expireCachedBodies',effective:{value:true}},
  {key:'maintenance.database.cachedBodyRetentionDays',effective:{value:60}},
 ] as AdminSetting[]});
});
it('loads the current administrator policy and runs a real maintenance batch',async()=>{
 renderInShell(()=><CacheMaintenance/>);
 expect(await screen.findByTestId('maintenance.database.cachedBodyRetentionDays')).toHaveTextContent('60');
 expect(screen.getByTestId('maintenance.database.expireCachedBodies')).toHaveTextContent('true');
 expect(listAdminSettings).toHaveBeenCalledTimes(1);
 expect(screen.getByRole('link',{name:'Inspect cache storage'})).toHaveAttribute('href','#/observability/database');
 fireEvent.click(screen.getByRole('button',{name:'Run a batch'}));
 await waitFor(()=>expect(adminRequest).toHaveBeenCalledWith('/maintenance/database-background',{method:'POST'}));
 await waitFor(()=>expect(loadDatabaseMaintenance).toHaveBeenCalledTimes(2));
});
it('keeps cleanup disabled for read-only maintenance operators',async()=>{
 const Ready=()=>{const shell=useShell();return <Show when={shell.can.grants().kind==='listed'}><CacheMaintenance/></Show>;};
 renderInShell(()=><Ready/>,{loadMe:holding(['maintenance.read'])});
 expect(await screen.findByRole('heading',{name:'Cache maintenance'})).toBeVisible();
 expect(screen.getByRole('button',{name:'Run a batch'})).toBeDisabled();
 expect(listAdminSettings).not.toHaveBeenCalled();expect(adminRequest).not.toHaveBeenCalled();
});
it('shows deferred sync-history cleanup even when body expiry completed',async()=>{
 vi.mocked(loadDatabaseMaintenance).mockResolvedValue({...report,
  cachedBodyExpiry:{status:'complete',at:1700000000,cutoffAt:1690000000,scannedMessages:10,initializedMessages:0,detachedMessages:0,removedBodies:0,removedBytes:0,protectedMessages:0,cursor:{scan:'',expire:''},elapsedMs:1},
  changeLogRetention:{at:1700000000,status:'wal-pressure',scannedRows:0,removedRows:0,oversizedBatches:0,elapsedMs:1},
 });
 renderInShell(()=><CacheMaintenance/>);
 expect(await screen.findByText('Maintenance yielded to a busy database, its work budget, or WAL pressure. It will retry automatically.')).toBeVisible();
 expect(screen.getByText('Sync-history rows removed: 0')).toBeVisible();
});
