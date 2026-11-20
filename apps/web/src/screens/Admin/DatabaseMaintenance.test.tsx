import {beforeEach,expect,it,vi} from 'vitest';
import {screen} from '@solidjs/testing-library';
import {DatabaseMaintenance} from './DatabaseMaintenance.tsx';
import {renderInShell} from './shell/testkit.tsx';
import {loadDatabaseMaintenance} from '../../api/admin/databaseMaintenance.ts';
vi.mock('../../api/admin/databaseMaintenance.ts',()=>({loadDatabaseMaintenance:vi.fn()}));
vi.mock('./OnlineDatabaseConversion.tsx',()=>({OnlineDatabaseConversion:()=> <p>Conversion controls</p>}));
vi.mock('./RetainedDatabases.tsx',()=>({RetainedDatabases:()=> <p>Retained database controls</p>}));
vi.mock('./BackgroundDatabaseMaintenance.tsx',()=>({BackgroundDatabaseMaintenance:()=> <p>Background maintenance controls</p>}));
beforeEach(()=>vi.mocked(loadDatabaseMaintenance).mockReset());
it('groups SQLite conversion, retirement and background cleanup under database maintenance',async()=>{
 vi.mocked(loadDatabaseMaintenance).mockResolvedValue({backend:'sqlite',running:false});
 renderInShell(()=><DatabaseMaintenance/>);
 expect(await screen.findByText('Conversion controls')).toBeVisible();
 expect(screen.getByText('Retained database controls')).toBeVisible();
 expect(screen.getByText('Background maintenance controls')).toBeVisible();
});
it('keeps PostgreSQL maintenance free of SQLite conversion or retirement controls',async()=>{
 vi.mocked(loadDatabaseMaintenance).mockResolvedValue({backend:'postgres',running:false});
 renderInShell(()=><DatabaseMaintenance/>);
 expect(await screen.findByText('Background maintenance controls')).toBeVisible();
 expect(screen.queryByText('Conversion controls')).toBeNull();
 expect(screen.queryByText('Retained database controls')).toBeNull();
});
