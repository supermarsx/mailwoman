import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {afterEach,describe,expect,it,vi} from 'vitest';
import {ApiError} from '../../api/client.ts';
import type {LinkedMailbox} from '../../api/linkedAccounts.ts';
import {RenewAccessDialog} from './RenewAccessDialog.tsx';

vi.mock('../../i18n/index.ts',()=>({t:(key:string,args?:Record<string,string>)=>args?.name!==undefined?`${key}:${args.name}`:key,isolate:(value:string)=>value,loadCatalog:async()=>undefined}));
vi.mock('../../api/publicConfig.ts',()=>({publicConfig:async()=>({antiBot:{mailbox:false}})}));
vi.mock('../../api/accountPreferenceLifecycle.ts',()=>({preferenceAccount:()=>({storageAuthority:'msa1_owner'}),preferenceAccountIsCurrent:()=>true}));
afterEach(cleanup);

const expired:LinkedMailbox={id:'child',accountId:'1',address:'child@example.org',displayName:'Child',status:'reauthRequired',revision:'v1',renewal:'signIn',renewalReason:'credentialRejected'};
const flush=async()=>{for(let i=0;i<6;i++)await Promise.resolve();};

describe('Renew access dialog (t48)',()=>{
  it('signs the same link in again with only the password and reports success',async()=>{
    const renewed:LinkedMailbox={...expired,status:'ready',renewal:'automatic',renewalReason:null};
    const api={renew:vi.fn(async()=>({account:renewed})),confirmRenew:vi.fn()};
    const onRenewed=vi.fn();
    render(()=><RenewAccessDialog account={expired} label="Child" api={api} onRenewed={onRenewed} onCancel={()=>undefined}/>);
    const dialog=screen.getByRole('dialog');
    expect(dialog).toHaveAccessibleName('linked-accounts-renew-title:Child');
    expect(dialog).toHaveTextContent('linked-accounts-renew-credential-rejected');
    expect(dialog.textContent).not.toMatch(/unlink/i);
    // The address is known: only the password is asked for.
    expect(screen.queryByLabelText('linked-accounts-address')).toBeNull();
    fireEvent.input(screen.getByLabelText('linked-accounts-password'),{target:{value:'new-secret'}});
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-renew-submit'}));
    await flush();
    expect(api.renew).toHaveBeenCalledExactlyOnceWith('child',{password:'new-secret'});
    expect(onRenewed).toHaveBeenCalledExactlyOnceWith(renewed);
  });
  it('asks for the sign-in name only for an old link with no known address',async()=>{
    const api={renew:vi.fn(async()=>({account:{...expired,status:'ready' as const}})),confirmRenew:vi.fn()};
    const {address:_drop,...legacy}={...expired,renewalReason:null};
    render(()=><RenewAccessDialog account={legacy} label="Child" api={api} onRenewed={()=>undefined} onCancel={()=>undefined}/>);
    fireEvent.input(screen.getByLabelText('linked-accounts-address'),{target:{value:' child@example.org '}});
    fireEvent.input(screen.getByLabelText('linked-accounts-password'),{target:{value:'pw'}});
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-renew-submit'}));
    await flush();
    expect(api.renew).toHaveBeenCalledExactlyOnceWith('child',{username:'child@example.org',password:'pw'});
  });
  it('a rejected password keeps the dialog open with a reason and clears the field',async()=>{
    const api={renew:vi.fn(async():Promise<never>=>{throw new ApiError(401,'no');}),confirmRenew:vi.fn()};
    const onRenewed=vi.fn();
    render(()=><RenewAccessDialog account={expired} label="Child" api={api} onRenewed={onRenewed} onCancel={()=>undefined}/>);
    fireEvent.input(screen.getByLabelText('linked-accounts-password'),{target:{value:'wrong'}});
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-renew-submit'}));
    await flush();
    expect(screen.getByRole('alert')).toHaveTextContent('linked-accounts-renew-password-rejected');
    expect((screen.getByLabelText('linked-accounts-password') as HTMLInputElement).value).toBe('');
    expect(onRenewed).not.toHaveBeenCalled();
  });
  it('a mailbox with two-step verification continues in the same dialog',async()=>{
    const api={
      renew:vi.fn(async()=>({twofaRequired:true as const,challengeId:'c1',factors:['totp']})),
      confirmRenew:vi.fn(async()=>({account:{...expired,status:'ready' as const}})),
    };
    render(()=><RenewAccessDialog account={{...expired,renewalReason:'factorRequired'}} label="Child" api={api as never} onRenewed={()=>undefined} onCancel={()=>undefined}/>);
    expect(screen.getByRole('dialog')).toHaveTextContent('linked-accounts-renew-factor');
    fireEvent.input(screen.getByLabelText('linked-accounts-password'),{target:{value:'pw'}});
    fireEvent.click(screen.getByRole('button',{name:'linked-accounts-renew-submit'}));
    await flush();
    expect(screen.getByRole('dialog')).toHaveTextContent('linked-accounts-factor-title');
  });
  it('Escape cancels',()=>{
    const onCancel=vi.fn();
    render(()=><RenewAccessDialog account={expired} label="Child" api={{renew:vi.fn(),confirmRenew:vi.fn()}} onRenewed={()=>undefined} onCancel={onCancel}/>);
    fireEvent.keyDown(screen.getByRole('dialog'),{key:'Escape'});
    expect(onCancel).toHaveBeenCalled();
  });
});
