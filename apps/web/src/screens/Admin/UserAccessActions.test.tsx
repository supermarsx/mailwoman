import {beforeEach,expect,it,vi} from 'vitest';
import {render,fireEvent,screen,waitFor,within} from '@solidjs/testing-library';
import {UserAccessActions} from './UserAccessActions.tsx';
import {adminRequest} from './shell/adminApi.ts';
import {loadCatalog} from '../../i18n/index.ts';
import type {UserSummary} from '../../state/slices/admin.ts';
vi.mock('./shell/adminApi.ts',async original=>({...await original<typeof import('./shell/adminApi.ts')>(),adminRequest:vi.fn()}));
const user:UserSummary={accountId:'opaque',username:'alice',domain:'example.test',quota:null,status:'active',flags:{disabled:false,forcePasswordChange:false,zeroAccess:false,remoteCacheWipe:false}};
beforeEach(async()=>{await loadCatalog('admin');vi.mocked(adminRequest).mockReset();vi.mocked(adminRequest).mockResolvedValue({status:200,etag:'"revision"',data:{status:'active',directoryAddress:'alice@example.test',deletedAt:null,canRestore:false}});});
it('requires typed local deletion confirmation and the loaded revision',async()=>{
 const changed=vi.fn(async()=>{});render(()=><UserAccessActions user={user} onChanged={changed}/>);fireEvent.click(screen.getByRole('button',{name:'Remove Mailwoman access for alice@example.test'}));
 const dialog=await screen.findByRole('alertdialog');expect(within(dialog).getByText(/upstream mailbox remain stored/)).toBeInTheDocument();const confirm=within(dialog).getByRole('button',{name:'Delete local access'});expect(confirm).toBeDisabled();expect(adminRequest).toHaveBeenCalledTimes(1);
 fireEvent.input(within(dialog).getByRole('textbox'),{target:{value:'alice@example.test'}});fireEvent.click(confirm);await waitFor(()=>expect(changed).toHaveBeenCalledOnce());expect(adminRequest).toHaveBeenLastCalledWith('/users/opaque/lifecycle',{method:'DELETE',ifMatch:'"revision"'});
});
it('blocks only after confirmation and preserves other flags',async()=>{
 const changed=vi.fn(async()=>{});render(()=><UserAccessActions user={user} onChanged={changed}/>);fireEvent.click(screen.getByRole('button',{name:'Block login for alice@example.test'}));const dialog=await screen.findByRole('alertdialog');fireEvent.click(within(dialog).getByRole('button',{name:'Block login'}));await waitFor(()=>expect(changed).toHaveBeenCalledOnce());expect(adminRequest).toHaveBeenLastCalledWith('/users/opaque/lifecycle',{method:'PUT',ifMatch:'"revision"',body:{disabled:true}});
});
it('cancel leaves access unchanged and deleted users cannot be restored',async()=>{
 const changed=vi.fn(async()=>{});const mounted=render(()=><UserAccessActions user={user} onChanged={changed}/>);fireEvent.click(screen.getByRole('button',{name:'Block login for alice@example.test'}));const dialog=await screen.findByRole('alertdialog');fireEvent.click(within(dialog).getByRole('button',{name:'Cancel'}));expect(adminRequest).toHaveBeenCalledTimes(1);expect(changed).not.toHaveBeenCalled();mounted.unmount();render(()=><UserAccessActions user={{...user,status:'deleted'}} onChanged={changed}/>);for(const button of screen.getAllByRole('button'))expect(button).toBeDisabled();
});
it('keeps a stale mutation failure visible without reporting success',async()=>{
 const changed=vi.fn(async()=>{});vi.mocked(adminRequest).mockResolvedValueOnce({status:200,etag:'"old"',data:{status:'active',directoryAddress:'alice@example.test'}}).mockRejectedValueOnce(new Error('Changed by another administrator'));
 render(()=><UserAccessActions user={user} onChanged={changed}/>);fireEvent.click(screen.getByRole('button',{name:'Block login for alice@example.test'}));const dialog=await screen.findByRole('alertdialog');fireEvent.click(within(dialog).getByRole('button',{name:'Block login'}));await screen.findByRole('alert');expect(changed).not.toHaveBeenCalled();expect(screen.getByRole('alertdialog')).toBeInTheDocument();
});

it('requires explicit revision-safe legacy clearing without claiming the directory is unblocked',async()=>{
 const changed=vi.fn(async()=>{});render(()=><UserAccessActions user={{...user,status:'disabled',flags:{...user.flags,disabled:true}}} onChanged={changed}/>);
 expect(screen.getByRole('button',{name:'Unblock login for alice@example.test'})).toBeDisabled();
 fireEvent.click(screen.getByRole('button',{name:'Clear legacy login block'}));const dialog=await screen.findByRole('alertdialog');
 expect(within(dialog).getByText(/does not guarantee that login is allowed/)).toBeInTheDocument();
 fireEvent.click(within(dialog).getByRole('button',{name:'Clear legacy block'}));await waitFor(()=>expect(changed).toHaveBeenCalledOnce());
 expect(adminRequest).toHaveBeenLastCalledWith('/users/opaque/legacy-block',{method:'DELETE',ifMatch:'"revision"'});
});
