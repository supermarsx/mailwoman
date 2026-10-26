import { describe, it, expect, vi } from 'vitest';
import { fireEvent, screen, within, waitFor } from '@solidjs/testing-library';
import { AutomaticLocks } from './AutomaticLocks.tsx';
import { renderInShell, holding } from './shell/testkit.tsx';
import type { AutomaticLock } from '../../api/admin/automaticLocks.ts';
const lock:AutomaticLock={surface:'admin',ip:'192.0.2.8',failures:5,lockedUntil:2000000000,revision:'rev1'};
describe('Temporary sign-in locks',()=>{
 it('opens the dedicated limit editor',async()=>{
 const {navigate}=renderInShell(()=><AutomaticLocks api={{list:async()=>[],unlock:vi.fn()}}/>);
 fireEvent.click(await screen.findByRole('button',{name:'Configure sign-in limits'}));
 expect(navigate).toHaveBeenCalledWith('/admin#/settings/security.signin.adminFailureLimit');
 });
 it('filters surfaces and requires confirmation before unlocking',async()=>{
 const list=vi.fn(async()=>[lock,{...lock,surface:'webmail' as const,ip:'192.0.2.9'}]);const unlock=vi.fn(async()=>undefined);
 renderInShell(()=><AutomaticLocks api={{list,unlock}}/>);
 await screen.findByText('192.0.2.8');
 fireEvent.change(screen.getByRole('combobox'),{target:{value:'webmail'}});
 expect(screen.queryByText('192.0.2.8')).toBeNull();
 fireEvent.click(screen.getByRole('button',{name:/Unlock Mailbox sign-in/}));
 expect(unlock).not.toHaveBeenCalled();
 fireEvent.click(within(screen.getByRole('alertdialog')).getByRole('button',{name:/^Unlock$/}));
 await waitFor(()=>expect(unlock).toHaveBeenCalledWith({...lock,surface:'webmail',ip:'192.0.2.9'}));
 });
 it('retains the row and confirmation after a stale revision rejection',async()=>{
 const unlock=vi.fn(async()=>{throw {status:412,code:'conflict'};});
 renderInShell(()=><AutomaticLocks api={{list:async()=>[lock],unlock}}/>);
 fireEvent.click(await screen.findByRole('button',{name:/Unlock Administrator sign-in/}));
 fireEvent.click(within(screen.getByRole('alertdialog')).getByRole('button',{name:/^Unlock$/}));
 await waitFor(()=>expect(unlock).toHaveBeenCalledWith(lock));
 expect(await screen.findByRole('alert')).toHaveTextContent(/changed|reload|refresh/i);
 expect(screen.getByText('192.0.2.8')).toBeInTheDocument();
 expect(screen.getByRole('alertdialog')).toBeInTheDocument();
 });
 it('bounds each page and hides mutations without bans.write',async()=>{
 renderInShell(()=><AutomaticLocks api={{list:async()=>Array.from({length:51},(_,i)=>({...lock,ip:`192.0.2.${i}`})),unlock:vi.fn()}}/>,{loadMe:holding(['bans.read'])});
 await screen.findByText('192.0.2.0');
 expect(screen.queryByText('192.0.2.50')).toBeNull();
 expect(screen.queryByRole('button',{name:/^Unlock/})).toBeNull();
 });
});
