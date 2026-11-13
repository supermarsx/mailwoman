import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor,within} from '@solidjs/testing-library';
import {MessageList} from './MessageList.tsx';
import {makeClient,mkEmail,MAILBOXES} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {createKeymap} from '../keymap/dispatcher.ts';
import * as quick from './messageQuickActions.ts';
import type {Email,Mailbox} from '../api/jmap-types.ts';

const printed=vi.hoisted(()=>vi.fn());
vi.mock('../viewers/printMessage.ts',()=>({printMessage:printed,printMessageDocument:()=>''}));
const apply=(values:Record<string,boolean>)=>loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:Object.fromEntries(Object.entries(values).map(([key,value])=>[key,{value,source:'global',locked:true}]))}),{headers:{'content-type':'application/json'}})});
beforeEach(()=>clearSettings());
afterEach(()=>{cleanup();clearSettings();localStorage.clear();vi.restoreAllMocks();printed.mockClear();});
async function open(values:Record<string,boolean>={},options:{message?:Partial<Email>;mailboxes?:Mailbox[]}={}){
 await apply(values);
 const app=createAppState(makeClient({emails:[mkEmail('m1',{subject:'Quarterly report',from:[{name:'Ann',email:'ann@example.org'}],...options.message})],mailboxes:options.mailboxes??[...MAILBOXES,{id:'projects',name:'Projects',parentId:null,role:null,sortOrder:5,totalEmails:0,unreadEmails:0}]}));
 render(()=><AppContext.Provider value={app}><MessageList/></AppContext.Provider>);
 await app.login({username:'me@example.org',password:'p'});await waitFor(()=>expect(app.messages()).toHaveLength(1));
 fireEvent.contextMenu(screen.getByText('Quarterly report').closest('button')!);
 return app;
}
describe('message context menu: reply, forward, print, copy',()=>{
 it('offers Reply, Reply all, Forward, Copy and Quick print next to the existing items',async()=>{
  await open();
  const labels=screen.getAllByRole('menuitem').map(item=>item.textContent);
  for(const label of ['Open message','Reply','Reply all','Forward','Copy','Quick print','Mark as'])expect(labels).toContain(label);
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

const junk:Mailbox={id:'junk',name:'Junk',parentId:null,role:'junk',sortOrder:4,totalEmails:1,unreadEmails:0};
const markMenu=()=>{fireEvent.click(screen.getByRole('menuitem',{name:'Mark as'}));return screen.getByRole('menu',{name:'Mark as'});};
describe('message context menu: Mark as submenu',()=>{
 it('groups status marks below one parent and keeps filing and destructive commands outside',async()=>{
  await open({}, {mailboxes:[...MAILBOXES,junk]});
  const outer=screen.getByRole('menu',{name:'More actions'}),parent=within(outer).getByRole('menuitem',{name:'Mark as'});
  expect(parent).toHaveAttribute('aria-haspopup','menu');
  for(const name of ['Mark unread','Flag','Mark as answered','Mark as forwarded','Mark as spam'])expect(within(outer).queryByRole('menuitem',{name})).toBeNull();
  for(const name of ['Archive','Delete','Pin'])expect(within(outer).getByRole('menuitem',{name})).toBeInTheDocument();
  const marks=markMenu();
  for(const name of ['Mark unread','Flag','Mark as answered','Mark as forwarded','Mark as spam'])expect(within(marks).getByRole('menuitem',{name})).toBeInTheDocument();
  expect(within(marks).queryByRole('menuitem',{name:'Archive'})).toBeNull();expect(within(marks).queryByRole('menuitem',{name:'Delete'})).toBeNull();
 });
 it('opens with the keyboard and returns focus to Mark as when the child menu closes',async()=>{
  await open();const parent=screen.getByRole('menuitem',{name:'Mark as'});parent.focus();fireEvent.keyDown(parent,{key:'ArrowRight'});
  const marks=screen.getByRole('menu',{name:'Mark as'});expect(within(marks).getByRole('menuitem',{name:'Mark unread'})).toHaveFocus();
  fireEvent.keyDown(marks,{key:'Escape'});expect(screen.queryByRole('menu',{name:'Mark as'})).toBeNull();expect(parent).toHaveFocus();expect(screen.getByRole('menu',{name:'More actions'})).toBeInTheDocument();
 });
 it('keeps the clicked-message target for read and flag toggles and current status labels',async()=>{
  const app=await open({}, {message:{keywords:{$seen:false,$flagged:true,$answered:true,$forwarded:true}}});
  const read=vi.spyOn(app,'markRead').mockResolvedValue(),flag=vi.spyOn(app,'flagMessage').mockResolvedValue();
  const marks=markMenu();
  expect(within(marks).getByRole('menuitem',{name:'Remove flag'})).toBeInTheDocument();
  expect(within(marks).getByRole('menuitem',{name:'Mark as unanswered'})).toBeInTheDocument();expect(within(marks).getByRole('menuitem',{name:'Mark as not forwarded'})).toBeInTheDocument();
  fireEvent.click(within(marks).getByRole('menuitem',{name:'Mark read'}));expect(read).toHaveBeenCalledWith('m1',true);
  fireEvent.contextMenu(screen.getByText('Quarterly report').closest('button')!);fireEvent.click(within(markMenu()).getByRole('menuitem',{name:'Remove flag'}));expect(flag).toHaveBeenCalledWith('m1',false);
 });
 it('keeps spam gating and executes the existing spam operation from the submenu',async()=>{
  const app=await open({}, {mailboxes:[...MAILBOXES,junk]}),spam=vi.spyOn(app,'markSpam').mockResolvedValue();
  fireEvent.click(within(markMenu()).getByRole('menuitem',{name:'Mark as spam'}));expect(spam).toHaveBeenCalledWith('m1');
  await apply({'mail.showSpam':false});fireEvent.contextMenu(screen.getByText('Quarterly report').closest('button')!);
  expect(within(markMenu()).queryByRole('menuitem',{name:'Mark as spam'})).toBeNull();
 });
 it('moves a junk message back to Inbox through the existing Not spam command',async()=>{
  const app=await open({}, {mailboxes:[...MAILBOXES,junk],message:{mailboxIds:{junk:true}}}),move=vi.spyOn(app,'moveMessage').mockResolvedValue();
  const marks=markMenu();expect(within(marks).queryByRole('menuitem',{name:'Mark as spam'})).toBeNull();
  fireEvent.click(within(marks).getByRole('menuitem',{name:'Not spam'}));expect(move).toHaveBeenCalledWith('m1',MAILBOXES.find(box=>box.role==='inbox')!.id);
 });
 it('disables the submenu while message actions are busy',async()=>{
  const app=await open();fireEvent.keyDown(screen.getByRole('menu',{name:'More actions'}),{key:'Escape'});app.listLoading=()=>true;
  fireEvent.contextMenu(screen.getByText('Quarterly report').closest('button')!);
  const parent=screen.getByRole('menuitem',{name:'Mark as'});expect(parent).toHaveAttribute('aria-disabled','true');fireEvent.click(parent);
  expect(screen.queryByRole('menu',{name:'Mark as'})).toBeNull();
 });
});

it('uses configured snooze choices and computes their delay when selected',async()=>{
 const app=await open();
 await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{'mail.snooze.presets':{value:[{id:'meeting',kind:'hours',amount:6,label:'After my meeting',enabled:true},{id:'hidden',kind:'hours',amount:2,label:'Hidden choice',enabled:false}],source:'user',locked:false}}}))});
 const save=vi.spyOn(app,'snoozeMessage').mockResolvedValue();const now=Date.now();
 fireEvent.click(screen.getByRole('menuitem',{name:'Snooze'}));
 expect(screen.getByRole('menuitem',{name:'Choose date and time…'})).toBeVisible();
 expect(screen.queryByRole('menuitem',{name:'Hidden choice'})).toBeNull();
 fireEvent.click(screen.getByRole('menuitem',{name:'After my meeting'}));
 expect(save).toHaveBeenCalledOnce();expect(save.mock.calls[0]?.[0]).toBe('m1');
 expect(new Date(save.mock.calls[0]![1]).getTime()).toBeGreaterThanOrEqual(now+6*3600000);
 expect(new Date(save.mock.calls[0]![1]).getTime()).toBeLessThan(now+6*3600000+10000);
});
