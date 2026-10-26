import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {afterEach,describe,expect,it,vi} from 'vitest';
import {LinkedAccountList} from './LinkedAccountList.tsx';
import type {LinkedMailboxList} from '../../api/linkedAccounts.ts';
vi.mock('../../i18n/index.ts',()=>({t:(key:string)=>key}));
afterEach(cleanup);
const value:LinkedMailboxList={primary:{accountId:'1',address:'primary@example.org',storageAuthority:'msa1_primary_verified'},accounts:[{id:'child',accountId:'1',address:'child@example.org',displayName:'Child',storageAuthority:'msa1_child_verified',status:'ready',revision:'v1',expiresAt:123}],policy:{enabled:true,maxAccounts:4,supportedAuthentication:['password']}};
function mount(data=value){
  const actions={onAdd:vi.fn(),onSwitch:vi.fn(),onUnlink:vi.fn()};
  render(()=><LinkedAccountList value={data} activeLinkId={null} busy={false} {...actions}/>);
  return actions;
}
describe('linked mailbox management',()=>{
  it('requires explicit unlink confirmation and passes the current revision row',()=>{
    const actions=mount();
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-unlink'}));
    expect(actions.onUnlink).not.toHaveBeenCalled();
    expect(screen.getByText('linked-accounts-unlink-help')).toBeVisible();
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-unlink-confirm'}));
    expect(actions.onUnlink).toHaveBeenCalledExactlyOnceWith(value.accounts[0]);
  });
  it('disables linking and child switching under policy while keeping unlink available',()=>{
    mount({...value,policy:{...value.policy,enabled:false}});
    expect(screen.getByRole('button',{name:'linked-accounts-add'})).toBeDisabled();
    expect(screen.getAllByRole('button',{name:'linked-accounts-open'})[1]).toBeDisabled();
    expect(screen.getByRole('button',{name:'linked-accounts-unlink'})).not.toBeDisabled();
  });
  it('shows the quota rather than offering an unusable add action',()=>{
    mount({...value,policy:{...value.policy,maxAccounts:1}});
    expect(screen.getByText('linked-accounts-limit')).toBeVisible();
    expect(screen.getByRole('button',{name:'linked-accounts-add'})).toBeDisabled();
  });
});
