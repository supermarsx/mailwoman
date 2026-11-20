import {expect,it,vi} from 'vitest';
import {createSignal,Show} from 'solid-js';
import {fireEvent,screen} from '@solidjs/testing-library';
import {Maintenance} from './Maintenance.tsx';
import {renderInShell,holding} from './shell/testkit.tsx';
import {useShell} from './shell/context.ts';
vi.mock('./DatabaseMaintenance.tsx',()=>({DatabaseMaintenance:()=> <p>Database maintenance panel</p>}));
vi.mock('./CacheMaintenance.tsx',()=>({CacheMaintenance:()=> <p>Cache maintenance panel</p>}));
vi.mock('./MaintenanceMode.tsx',()=>({MaintenanceMode:()=> <p>Maintenance mode panel</p>}));
vi.mock('./CleanupMaintenance.tsx',()=>({CleanupMaintenance:()=> <p>Cleanup jobs panel</p>}));
vi.mock('./RethreadMaintenance.tsx',()=>({RethreadMaintenance:()=> <p>Rethread jobs panel</p>}));
it('navigates between dedicated database and cache maintenance tabs',async()=>{
 const [route,setRoute]=createSignal({section:'maintenance',rest:['database']});
 const {navigate}=renderInShell(()=><Maintenance/>,{route,navigate:path=>setRoute({section:'maintenance',rest:path.split('#/maintenance/')[1]?.split('/')??[]})});
 expect(await screen.findByText('Database maintenance panel')).toBeVisible();
 fireEvent.click(screen.getByRole('link',{name:'Cache'}));
 expect(navigate).toHaveBeenCalledWith('/admin#/maintenance/cache');
 expect(await screen.findByText('Cache maintenance panel')).toBeVisible();
 expect(screen.queryByText('Database maintenance panel')).toBeNull();
 expect(screen.getByRole('link',{name:'Cache'})).toHaveAttribute('aria-current','page');
});
it('keeps database and cache maintenance available to read-only operators without repair actions',async()=>{
 const Ready=()=>{const shell=useShell();return <Show when={shell.can.grants().kind==='listed'}><Maintenance/></Show>;};
 renderInShell(()=><Ready/>,{route:()=>({section:'maintenance',rest:['cache']}),loadMe:holding(['maintenance.read'])});
 expect(await screen.findByText('Cache maintenance panel')).toBeVisible();
 expect(screen.getByRole('link',{name:'Database'})).toBeVisible();
 expect(screen.queryByRole('link',{name:'Repair jobs'})).toBeNull();
});
