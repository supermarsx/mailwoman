import {expect,it,vi} from 'vitest';
import {fireEvent,screen,waitFor} from '@solidjs/testing-library';
import {BackgroundDatabaseMaintenance} from './BackgroundDatabaseMaintenance.tsx';
import {renderInShell,holding} from './shell/testkit.tsx';
import {adminRequest} from './shell/adminApi.ts';
vi.mock('./shell/adminApi.ts',()=>({adminRequest:vi.fn()}));
it('runs the bounded action and explains the explicit offline conversion',async()=>{
 vi.mocked(adminRequest).mockResolvedValue({data:{running:false,lastReport:{at:1700000000,status:'complete',vacuumStatus:'offline-conversion-required',scanned:64,marked:10,removed:2,removedBytes:1024,reclaimedPages:0}},etag:null,status:200});
 renderInShell(()=><BackgroundDatabaseMaintenance report={{running:false}}/>);
 const button=await screen.findByRole('button',{name:'Run a batch'});
 await waitFor(()=>expect(button).not.toBeDisabled());fireEvent.click(button);
 await waitFor(()=>expect(adminRequest).toHaveBeenCalledWith('/maintenance/database-background',{method:'POST'}));
 expect(await screen.findByText('./deploy.sh maintenance --apply --enable-incremental-vacuum')).toBeVisible();
 expect(screen.getByRole('link').getAttribute('href')).toBe('#/settings/maintenance.database.enabled');
});
it('respects maintenance capability restrictions',async()=>{
 renderInShell(()=><BackgroundDatabaseMaintenance report={{running:false}}/>,{loadMe:holding(['observability.read'])});
 expect(await screen.findByRole('button',{name:'Run a batch'})).toBeDisabled();
});
