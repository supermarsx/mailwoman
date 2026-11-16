import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,render,screen} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {MailboxConnectionStatus} from './MailboxConnectionStatus.tsx';
import {clearAccountHealth,reportAccountHealth,reconcileLinkedHealth,accountHealth} from '../../realtime/accountHealth.ts';
import {setPrimaryMailboxAuthority} from '../../api/linkedMailboxSession.ts';
const config=vi.hoisted(()=>({read:()=>({} as Record<string,{value:unknown;locked:boolean}>)}));
vi.mock('../../api/settings.ts',()=>({settings:()=>config.read()}));
vi.mock('../../i18n/index.ts',()=>({t:(key:string)=>key,loadCatalog:async()=>{}}));
afterEach(()=>{cleanup();clearAccountHealth();setPrimaryMailboxAuthority(undefined);config.read=()=>({});});
const mailbox={username:'child@example.org',accountId:'child',storageAuthority:'child-authority'};
it('is off until the effective policy enables it and reacts to locked admin changes',()=>{
 const [map,setMap]=createSignal<Record<string,{value:unknown;locked:boolean}>>({});config.read=map;
 render(()=><MailboxConnectionStatus linkId="child"/>);expect(screen.queryByRole('img')).toBeNull();
 setMap({'accounts.showConnectionStatus':{value:true,locked:true}});expect(screen.getByRole('img')).toHaveAttribute('data-mailbox-connection','unknown');
 setMap({'accounts.showConnectionStatus':{value:false,locked:true}});expect(screen.queryByRole('img')).toBeNull();
});
it('shows only health owned by the current primary and corresponding mailbox',()=>{
 config.read=()=>({'accounts.showConnectionStatus':{value:true,locked:false}});
 setPrimaryMailboxAuthority('other');reportAccountHealth(mailbox,'child','offline');
 setPrimaryMailboxAuthority('owner');render(()=><MailboxConnectionStatus linkId="child"/>);
 expect(screen.getByRole('img')).toHaveAttribute('data-mailbox-connection','unknown');
 reportAccountHealth(mailbox,'child','online');expect(screen.getByRole('img')).toHaveAttribute('data-tooltip','common-conn-account-online');
 reportAccountHealth(mailbox,'child','auth-expired');expect(screen.getByRole('img')).toHaveAttribute('data-tooltip','common-conn-account-auth-expired');
});
it('keeps readable status available in settings while icons are disabled',()=>{
 setPrimaryMailboxAuthority('owner');reportAccountHealth(mailbox,'child','offline');
 render(()=><MailboxConnectionStatus linkId="child" detail/>);
 expect(screen.getByText('common-conn-account-offline')).toBeVisible();expect(screen.queryByRole('img')).toBeNull();
});
it('does not claim a listed credential has a healthy upstream connection',()=>{
 const linked={id:'child',...mailbox,displayName:'Child',status:'ready'};
 setPrimaryMailboxAuthority('owner');reconcileLinkedHealth('owner',[linked]);expect(accountHealth()[0]?.state).toBe('unknown');
 reportAccountHealth(mailbox,'child','degraded');reconcileLinkedHealth('owner',[linked]);expect(accountHealth()[0]?.state).toBe('degraded');
 reconcileLinkedHealth('owner',[{...linked,status:'reauthRequired'}]);expect(accountHealth()[0]?.state).toBe('auth-expired');
 reconcileLinkedHealth('owner',[linked]);expect(accountHealth()[0]?.state).toBe('unknown');
});
