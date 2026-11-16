import {describe,it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {AppContext} from '../../state/context.ts';
import type {AppState} from '../../state/store.ts';
import {NotificationAccountPicker} from './NotificationAccountPicker.tsx';
const list=vi.hoisted(()=>vi.fn(async()=>({primary:{address:'primary@example.test'},policy:{enabled:true},accounts:[{id:'child',address:'child@example.test',status:'ready'}]})));
vi.mock('../../api/linkedAccounts.ts',()=>({createLinkedAccountsApi:()=>({list,dispose:vi.fn()})}));
vi.mock('../../shell/navigationGuard.ts',()=>({confirmShellNavigation:async()=>true}));
describe('notification account selection',()=>{
 it('switches registration context without navigating away and waits for pending saves',async()=>{
  const change=vi.fn(async()=>{});const [disabled,setDisabled]=createSignal(true);
  const app={switchingMailbox:()=>false,linkedMailboxId:()=>null,switchLinkedMailbox:change} as unknown as AppState;
  render(()=><AppContext.Provider value={app}><NotificationAccountPicker disabled={disabled()}/></AppContext.Provider>);
  const trigger=await screen.findByRole('button',{name:/primary@example.test/});
  expect(trigger).toBeDisabled();setDisabled(false);await waitFor(()=>expect(trigger).toBeEnabled());
  fireEvent.click(trigger);fireEvent.click(await screen.findByRole('menuitemradio',{name:/child@example.test/}));
  await waitFor(()=>expect(change).toHaveBeenCalledWith('child'));
 });
});
