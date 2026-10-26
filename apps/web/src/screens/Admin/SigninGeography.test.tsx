import { describe,it,expect,vi } from 'vitest';
import { screen,fireEvent,waitFor } from '@solidjs/testing-library';
import { SigninGeographyPage } from './SigninGeography.tsx';
import { renderInShell,holding } from './shell/testkit.tsx';
import type { SigninGeography } from '../../api/admin/signinGeography.ts';
const report:SigninGeography={sourceIp:'192.0.2.8',country:null,database:'unavailable',scope:'off',mode:'allow',countries:['PT'],unknown:'deny',administratorDecision:'off',mailboxDecision:'off',scopeDescription:'new-sign-ins-only'};
describe('Sign-in geography diagnostic',()=>{
 it('distinguishes unavailable database from an off policy and links the authoritative editor',async()=>{
 const {navigate}=renderInShell(()=><SigninGeographyPage load={async()=>report}/>);
 expect(await screen.findByText('192.0.2.8')).toBeInTheDocument();
 expect(screen.getByText(/local country database is missing or unreadable/)).toBeInTheDocument();
 expect(screen.getByText(/existing sessions are not revoked/)).toBeInTheDocument();
 fireEvent.click(screen.getByRole('button',{name:'Configure country policy'}));
 expect(navigate).toHaveBeenCalledWith('/admin#/settings/security.signin.geoScope');
 });
 it('reloads actual decisions after a policy change',async()=>{
 const load=vi.fn().mockResolvedValueOnce(report).mockResolvedValueOnce({...report,database:'ready',country:'PT',scope:'all',administratorDecision:'allow',mailboxDecision:'allow'});
 renderInShell(()=><SigninGeographyPage load={load}/>);
 await screen.findByText('192.0.2.8');fireEvent.click(screen.getByRole('button',{name:'Refresh diagnostic'}));
 await waitFor(()=>expect(screen.queryByText(/local country database is missing or unreadable/)).toBeNull());
 expect(load).toHaveBeenCalledTimes(2);
 });
 it('offers retry without configuration controls to a read-only operator',async()=>{
 renderInShell(()=><SigninGeographyPage load={async()=>{throw {status:503};}}/>,{loadMe:holding(['bans.read'])});
 expect(await screen.findByRole('alert')).toHaveTextContent('503');
 expect(screen.queryByRole('button',{name:'Configure country policy'})).toBeNull();
 });
});
