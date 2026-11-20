import {beforeEach,expect,it,vi} from 'vitest';
import {fireEvent,screen,waitFor,within} from '@solidjs/testing-library';
import {RetainedDatabases} from './RetainedDatabases.tsx';
import {loadRetainedDatabases,retireDatabase,type RetainedDatabaseReport} from '../../api/admin/retainedDatabases.ts';
import {renderInShell,holding} from './shell/testkit.tsx';
import {expectNoAxeViolations,expectNoInlineStyle} from './shell/uxTestkit.ts';
vi.mock('../../api/admin/retainedDatabases.ts',()=>({loadRetainedDatabases:vi.fn(),retireDatabase:vi.fn()}));
const report=():RetainedDatabaseReport=>({supported:true,activeGeneration:'active-generation',observedAt:1700000000,retained:[{generationId:'legacy',fileName:'mailwoman.db',path:'/data/mailwoman.db',bytes:1073741824,modifiedAt:1690000000,fileIdentity:'verified-file-identity',deletable:true,blockedReason:null}]});
beforeEach(()=>{
 vi.mocked(loadRetainedDatabases).mockReset().mockResolvedValue(report());
 vi.mocked(retireDatabase).mockReset().mockResolvedValue({generationId:'legacy',fileName:'mailwoman.db',removedBytes:1073741824});
});
it('requires explicit typed confirmation and binds deletion to the displayed file and active generation',async()=>{
 const {container}=renderInShell(()=><RetainedDatabases/>);
 const remove=await screen.findByRole('button',{name:'Delete mailwoman.db'});
 expect(screen.getByText('/data/mailwoman.db')).toBeVisible();
 await expectNoAxeViolations(container);expectNoInlineStyle(container);
 fireEvent.click(remove);
 const dialog=await screen.findByRole('alertdialog');
 const confirm=within(dialog).getByRole('button',{name:'Delete database copy'});
 expect(confirm).toBeDisabled();
 fireEvent.input(within(dialog).getByRole('textbox'),{target:{value:'some-other.db'}});expect(confirm).toBeDisabled();
 expect(retireDatabase).not.toHaveBeenCalled();
 fireEvent.input(within(dialog).getByRole('textbox'),{target:{value:'mailwoman.db'}});
 await waitFor(()=>expect(confirm).not.toBeDisabled());
 const empty=report();empty.retained=[];vi.mocked(loadRetainedDatabases).mockResolvedValue(empty);
 fireEvent.click(confirm);
 await waitFor(()=>expect(retireDatabase).toHaveBeenCalledWith({generationId:'legacy',expectedActiveGeneration:'active-generation',fileIdentity:'verified-file-identity',confirmation:'mailwoman.db'}));
 await screen.findByText('No verified retained database is available.');
 expect(screen.queryByRole('alertdialog')).toBeNull();
 expect(await screen.findByText('Deleted retained database mailwoman.db.')).toBeVisible();
});
it('does not delete when the dialog is cancelled',async()=>{
 renderInShell(()=><RetainedDatabases/>);fireEvent.click(await screen.findByRole('button',{name:'Delete mailwoman.db'}));
 const dialog=await screen.findByRole('alertdialog');fireEvent.click(within(dialog).getByRole('button',{name:'Cancel'}));
 await waitFor(()=>expect(screen.queryByRole('alertdialog')).toBeNull());expect(retireDatabase).not.toHaveBeenCalled();
});
it('keeps read-only administrators and sidecar-bearing copies protected',async()=>{
 const value=report();value.retained[0]!.deletable=false;value.retained[0]!.blockedReason='sqlite-sidecars';vi.mocked(loadRetainedDatabases).mockResolvedValue(value);
 renderInShell(()=><RetainedDatabases/>,{loadMe:holding(['maintenance.read'])});
 expect(await screen.findByRole('button',{name:'Delete mailwoman.db'})).toBeDisabled();
 expect(screen.getByText(/SQLite recovery files are still present/)).toBeVisible();expect(retireDatabase).not.toHaveBeenCalled();
});
it('keeps a refused deletion open and never reports success',async()=>{
 vi.mocked(retireDatabase).mockRejectedValue(new Error('Retained database changed'));
 renderInShell(()=><RetainedDatabases/>);fireEvent.click(await screen.findByRole('button',{name:'Delete mailwoman.db'}));
 const dialog=await screen.findByRole('alertdialog');fireEvent.input(within(dialog).getByRole('textbox'),{target:{value:'mailwoman.db'}});
 fireEvent.click(within(dialog).getByRole('button',{name:'Delete database copy'}));
 await waitFor(()=>expect(retireDatabase).toHaveBeenCalledTimes(1));
 await waitFor(()=>expect(within(dialog).getByRole('button',{name:'Delete database copy'})).not.toBeDisabled());
 expect(screen.getByRole('alertdialog')).toBeVisible();expect(screen.queryByText('Deleted retained database mailwoman.db.')).toBeNull();
});
