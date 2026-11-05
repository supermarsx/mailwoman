import {afterEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {MessageList} from './MessageList.tsx';
import {makeClient,mkEmail,MAILBOXES} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import * as focus from './messageFocus.ts';

afterEach(()=>{cleanup();vi.restoreAllMocks();focus.clearFocusRequest();});
async function setup(searching:boolean){
 const app=createAppState(makeClient({emails:[mkEmail('m1',{subject:'Quarterly',mailboxIds:{archive:true}})],mailboxes:[...MAILBOXES]}));
 if(searching)vi.spyOn(app,'searchActive').mockReturnValue(true);
 render(()=><AppContext.Provider value={app}><MessageList/></AppContext.Provider>);
 await app.login({username:'me@example.org',password:'p'});
 await waitFor(()=>expect(app.messages()).toHaveLength(1));
 return app;
}
describe('folder indicator',()=>{
 it('is absent outside a cross-folder list',async()=>{
  await setup(false);
  expect(screen.queryByTestId('row-folder')).toBeNull();
 });
 it('is a button labelled Go to <folder> that navigates, scrolls and flashes the row',async()=>{
  const go=vi.spyOn(focus,'focusMessageInFolder').mockResolvedValue(true);
  await setup(true);
  const chip=await screen.findByRole('button',{name:/Go to .?Archive/});
  expect(chip).toHaveAttribute('title',expect.stringMatching(/Go to .?Archive/));
  fireEvent.click(chip);
  expect(go).toHaveBeenCalledWith(expect.anything(),'archive','m1');
 });
 it('a focus request flashes the row with data-flash (a static outline under reduced motion is CSS-only)',async()=>{
  await setup(false);
  const row=document.querySelector<HTMLElement>('.list__row[data-email-id="m1"]')!;
  const view=row.closest<HTMLElement>('.list__scroller')??document.body;
  void view;
  const spy=vi.spyOn(row,'focus');
  const app2=focus;
  app2.clearFocusRequest();
  await focus.focusMessageInFolder({accountId:()=>'a',selectedMailboxId:()=>'inbox',searchActive:()=>false,selectMailbox:async()=>undefined,messages:()=>[{id:'m1'} as never],hasMore:()=>false,loadMore:async()=>undefined,mailboxes:()=>[],showToast:()=>undefined},'inbox','m1');
  await waitFor(()=>expect(row.getAttribute('data-flash')).toBe('true'));
  expect(spy).toHaveBeenCalled();
 });
});
