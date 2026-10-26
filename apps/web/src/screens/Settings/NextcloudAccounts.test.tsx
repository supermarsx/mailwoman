import {describe,it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {NextcloudAccounts} from './NextcloudAccounts.tsx';
import type {NextcloudAccountsApi} from '../../api/nextcloudAccounts.ts';
function service():NextcloudAccountsApi{return {
  account:vi.fn(async()=>({accountId:'mailbox-a',instances:[{id:'cloud',name:'Company cloud',baseUrl:'https://cloud.test/',enabled:true,etag:'"instance"'}],links:[],maxFileBytes:8388608})),
  link:vi.fn(async()=>({instanceId:'cloud',username:'alice',ready:true,etag:'"link"'})),
  unlink:vi.fn(async()=>{}),instances:vi.fn(async()=>({accountId:'mailbox-a',instances:[],maxInstances:64,maxFileBytes:8388608})),save:vi.fn(),
};}
describe('account-owned Nextcloud editor',()=>{
  it('uses a dedicated editor, verifies once and clears a rejected password without replay',async()=>{
    const api=service();vi.mocked(api.link).mockRejectedValue(new Error('rejected'));
    render(()=><NextcloudAccounts api={api}/>);
    fireEvent.click(await screen.findByRole('button',{name:'Link account'}));
    expect(screen.queryByRole('heading',{name:'Nextcloud accounts'})).not.toBeInTheDocument();
    const username=screen.getByLabelText('Nextcloud app-password username');const password=screen.getByLabelText('Nextcloud app password');
    fireEvent.input(username,{target:{value:'alice'}});fireEvent.input(password,{target:{value:'app-secret'}});
    fireEvent.click(screen.getByRole('button',{name:'Verify and link'}));
    await screen.findByRole('alert');expect(api.link).toHaveBeenCalledTimes(1);expect(username).toHaveValue('alice');expect(password).toHaveValue('');
    await waitFor(()=>expect(screen.getByRole('button',{name:'Verify and link'})).toBeEnabled());expect(api.link).toHaveBeenCalledTimes(1);
  });
  it('never selects an unapproved URL and unlinks only the displayed account link revision',async()=>{
    const api=service();vi.mocked(api.account).mockResolvedValue({accountId:'mailbox-a',instances:[{id:'cloud',name:'Company cloud',baseUrl:'https://cloud.test/',enabled:true,etag:'"instance"'}],links:[{instanceId:'cloud',username:'alice',ready:false,etag:'"old-link"'}],maxFileBytes:8388608});
    render(()=><NextcloudAccounts api={api}/>);
    fireEvent.click(await screen.findByRole('button',{name:'Unlink account'}));
    await waitFor(()=>expect(api.unlink).toHaveBeenCalledWith('cloud','mailbox-a','"old-link"'));
    expect(screen.queryByRole('textbox')).not.toBeInTheDocument();
  });
});
