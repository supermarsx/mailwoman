import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {MessageList} from './MessageList.tsx';
import {makeClient,mkEmail,MAILBOXES} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {createKeymap} from '../keymap/dispatcher.ts';
import * as quick from './messageQuickActions.ts';

const printed=vi.hoisted(()=>vi.fn());
vi.mock('../viewers/printMessage.ts',()=>({printMessage:printed,printMessageDocument:()=>''}));
const apply=(values:Record<string,boolean>)=>loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:Object.fromEntries(Object.entries(values).map(([key,value])=>[key,{value,source:'global',locked:true}]))}),{headers:{'content-type':'application/json'}})});
beforeEach(()=>clearSettings());
afterEach(()=>{cleanup();clearSettings();localStorage.clear();vi.restoreAllMocks();printed.mockClear();});
async function open(values:Record<string,boolean>={}){
 await apply(values);
 const app=createAppState(makeClient({emails:[mkEmail('m1',{subject:'Quarterly report',from:[{name:'Ann',email:'ann@example.org'}]})],mailboxes:[...MAILBOXES,{id:'projects',name:'Projects',parentId:null,role:null,sortOrder:5,totalEmails:0,unreadEmails:0}]}));
 render(()=><AppContext.Provider value={app}><MessageList/></AppContext.Provider>);
 await app.login({username:'me@example.org',password:'p'});await waitFor(()=>expect(app.messages()).toHaveLength(1));
 fireEvent.contextMenu(screen.getByText('Quarterly report').closest('button')!);
 return app;
}
describe('message context menu: reply, forward, print, copy',()=>{
 it('offers Reply, Reply all, Forward, Copy and Quick print next to the existing items',async()=>{
  await open();
  const labels=screen.getAllByRole('menuitem').map(item=>item.textContent);
  for(const label of ['Open message','Reply','Reply all','Forward','Copy','Quick print','Flag'])expect(labels).toContain(label);
  expect(labels.indexOf('Reply')).toBeLessThan(labels.indexOf('Copy'));
 });
 it('Reply opens the message and runs the reader\'s own reply action',async()=>{
  const app=await open();const run=vi.fn(()=>true);
  const keymap=createKeymap({target:document});vi.spyOn(await import('../keymap/index.ts'),'getKeymap').mockReturnValue({...keymap,run} as never);
  vi.spyOn(app,'openMessage').mockImplementation(async()=>{vi.spyOn(app,'openEmail').mockReturnValue(mkEmail('m1'));});
  fireEvent.click(screen.getByRole('menuitem',{name:'Reply all'}));
  await waitFor(()=>expect(run).toHaveBeenCalledWith('replyAll'));
 });
 it('the mail.allowReply and mail.allowForward switches disable the entries with the feature-off tooltip',async()=>{
  await open({'mail.allowReply':false,'mail.allowForward':false,'mail.allowPrint':false});
  for(const name of ['Reply','Reply all','Forward','Quick print']){const item=screen.getByRole('menuitem',{name});expect(item).toHaveAttribute('aria-disabled','true');expect(item.getAttribute('title')).toBeTruthy();}
 });
 it('Copy offers sender, subject and a folder list that excludes the message\'s own folder',async()=>{
  await open();fireEvent.click(screen.getByRole('menuitem',{name:'Copy'}));
  expect(await screen.findByRole('menuitem',{name:'Copy sender address'})).toBeInTheDocument();expect(screen.getByRole('menuitem',{name:'Copy subject'})).toBeInTheDocument();
  fireEvent.click(screen.getByRole('menuitem',{name:'Copy message to folder…'}));
  const folders=(await screen.findAllByRole('menuitem')).map(item=>item.textContent);
  expect(folders).toContain('Projects');expect(folders).not.toContain('Inbox');
 });
 it('Copy sender address writes the address to the clipboard',async()=>{
  const write=vi.fn(async()=>undefined);Object.defineProperty(navigator,'clipboard',{value:{writeText:write},configurable:true});
  await open();fireEvent.click(screen.getByRole('menuitem',{name:'Copy'}));fireEvent.click(await screen.findByRole('menuitem',{name:'Copy sender address'}));
  await waitFor(()=>expect(write).toHaveBeenCalledWith('ann@example.org'));
 });
 it('Quick print prints the message without opening it',async()=>{
  const spy=vi.spyOn(quick,'quickPrint').mockResolvedValue();const app=await open();const opened=vi.spyOn(app,'openMessage');
  fireEvent.click(screen.getByRole('menuitem',{name:'Quick print'}));
  expect(spy).toHaveBeenCalledWith(expect.anything(),'m1');expect(opened).not.toHaveBeenCalled();
 });
});
