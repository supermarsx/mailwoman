import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {AppContext} from '../state/context.ts';
import {createAppState} from '../state/store.ts';
import {makeClient,mkEmail,MAILBOXES} from '../components/appHarness.tsx';
import {MailboxScreen} from './Mailbox.tsx';
import {RealtimeContext} from '../realtime/context.ts';
import {createRealtimeController} from '../realtime/controller.ts';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {accountUiPreferences} from '../api/accountPreferences.ts';
import {WORKSPACE_STORAGE_KEY} from '../realtime/workspaceStorage.ts';
import type {Mailbox} from '../api/jmap-types.ts';

// The sidebar's Create folder button (mail.showCreateFolderButton) and the nested folder tree.
const TREE:Mailbox[]=[...MAILBOXES,
 {id:'work',name:'Work',parentId:null,role:null,sortOrder:10,totalEmails:5,unreadEmails:1},
 {id:'q3',name:'Quarter 3',parentId:'work',role:null,sortOrder:10,totalEmails:2,unreadEmails:4},
 {id:'rep',name:'Reports',parentId:'q3',role:null,sortOrder:10,totalEmails:1,unreadEmails:0},
 {id:'home',name:'Home',parentId:null,role:null,sortOrder:11,totalEmails:0,unreadEmails:0},
];
const apply=(values:Record<string,boolean>)=>loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:Object.fromEntries(Object.entries(values).map(([key,value])=>[key,{value,source:'user',locked:false}]))}),{headers:{'content-type':'application/json'}})});
beforeEach(async()=>{suspendAccountPreferences();clearSettings();localStorage.removeItem(WORKSPACE_STORAGE_KEY);history.replaceState(null,'','#/mail');await new Promise(resolve=>setTimeout(resolve,0));});
afterEach(()=>{cleanup();clearSettings();});
async function mount(values:Record<string,boolean>={},boxes:Mailbox[]=TREE){
 const app=createAppState(makeClient({emails:[mkEmail('a')],mailboxes:boxes}));const realtime=createRealtimeController();
 await apply(values);await app.login({username:'me@example.org',password:'p'});
 render(()=><AppContext.Provider value={app}><RealtimeContext.Provider value={realtime}><MailboxScreen/></RealtimeContext.Provider></AppContext.Provider>);
 await screen.findByText('Subject a',{selector:'.list__subject'});
 return app;
}
const sidebar=()=>document.querySelector('aside.sidebar') as HTMLElement;

describe('Create folder button next to Compose',()=>{
 it('is absent by default',async()=>{
  await mount();
  expect(sidebar().querySelector('.sidebar__create-folder')).toBeNull();
  expect(sidebar().querySelector('.sidebar__compose')).not.toBeNull();
 });
 it('appears icon-only, named and with a tooltip, once mail.showCreateFolderButton is on',async()=>{
  await mount({'mail.showCreateFolderButton':true});
  const button=await waitFor(()=>{const node=sidebar().querySelector<HTMLButtonElement>('.sidebar__create-folder');expect(node).not.toBeNull();return node!;});
  expect(button).toHaveAttribute('aria-label','Create folder');expect(button).toHaveAttribute('title','Create folder');
  expect(button.textContent?.trim()).toBe('');
  expect(button.previousElementSibling?.querySelector('.sidebar__compose')??button.parentElement?.querySelector('.sidebar__compose')).not.toBeNull();
 });
});

describe('nested folder tree in the sidebar',()=>{
 const names=()=>Array.from(sidebar().querySelectorAll('[data-folder-region] .sidebar__box:not([data-folder-disclosure]) .sidebar__box-name')).map(node=>node.textContent);
 it('shows subfolders at any depth under their parent, with twisties for parents only',async()=>{
  await mount();
  await waitFor(()=>expect(names()).toEqual(expect.arrayContaining(['Work','Quarter 3','Reports','Home'])));
  expect(names().indexOf('Work')).toBeLessThan(names().indexOf('Quarter 3'));expect(names().indexOf('Quarter 3')).toBeLessThan(names().indexOf('Reports'));expect(names().indexOf('Reports')).toBeLessThan(names().indexOf('Home'));
  const rows=Array.from(sidebar().querySelectorAll<HTMLElement>('[style*="--folder-depth"]'));
  expect(rows.map(row=>row.style.getPropertyValue('--folder-depth'))).toEqual(expect.arrayContaining(['0','1','2']));
  expect(sidebar().querySelectorAll('.sidebar__twisty')).toHaveLength(2);
 });
 it('collapses and expands a folder, persisting the choice, and rolls the unread count up while collapsed',async()=>{
  await mount();
  await waitFor(()=>expect(names()).toContain('Quarter 3'));
  fireEvent.click(await screen.findByRole('button',{name:'Hide subfolders of Work'}));
  await waitFor(()=>expect(names()).not.toContain('Quarter 3'));
  expect(accountUiPreferences().folderCollapsed).toEqual(['work']);
  const work=screen.getByRole('button',{name:'Work'});expect(work).toHaveAttribute('aria-expanded','false');expect(work.textContent).toContain('5');
  fireEvent.click(screen.getByRole('button',{name:'Show subfolders of Work'}));
  await waitFor(()=>expect(names()).toContain('Quarter 3'));expect(accountUiPreferences().folderCollapsed).toEqual([]);
 });
 it('Right and Left arrows expand and collapse the focused treeitem; Right again enters, Down moves on, Left goes to the parent',async()=>{
  await mount();await waitFor(()=>expect(names()).toContain('Quarter 3'));
  const item=(name:string)=>screen.getByRole('treeitem',{name});
  item('Work').focus();
  fireEvent.keyDown(item('Work'),{key:'ArrowLeft'});await waitFor(()=>expect(names()).not.toContain('Quarter 3'));
  expect(item('Work')).toHaveAttribute('aria-expanded','false');
  fireEvent.keyDown(item('Work'),{key:'ArrowRight'});await waitFor(()=>expect(names()).toContain('Quarter 3'));
  expect(item('Work')).toHaveAttribute('aria-expanded','true');
  fireEvent.keyDown(item('Work'),{key:'ArrowRight'});expect(document.activeElement).toBe(item('Quarter 3'));
  fireEvent.keyDown(item('Quarter 3'),{key:'ArrowDown'});expect(document.activeElement).toBe(item('Reports'));
  fireEvent.keyDown(item('Reports'),{key:'ArrowLeft'});expect(document.activeElement).toBe(item('Quarter 3'));
  fireEvent.keyDown(item('Quarter 3'),{key:'ArrowLeft'});await waitFor(()=>expect(names()).not.toContain('Reports'));
  fireEvent.keyDown(item('Quarter 3'),{key:'ArrowLeft'});expect(document.activeElement).toBe(item('Work'));
 });
 it('orders user folders by the saved per-parent choice',async()=>{
  const app=await mount();void app;
  const {setAccountUiPreferences}=await import('../api/accountPreferences.ts');
  setAccountUiPreferences({folderSort:{_top:{by:'name',dir:'desc'}}});
  await waitFor(()=>{const list=names();expect(list.indexOf('Work')).toBeLessThan(list.indexOf('Home'));});
  setAccountUiPreferences({folderSort:{_top:{by:'name',dir:'asc'}}});
  await waitFor(()=>{const list=names();expect(list.indexOf('Home')).toBeLessThan(list.indexOf('Work'));});
 });
});
void vi;

describe('secondary divider before views and queues',()=>{
 const divider=()=>sidebar().querySelector('[role=separator].sidebar__divider-secondary');
 it('sits between the user folders and the Attachments/Outbox group, named for screen readers',async()=>{
  await mount();
  await waitFor(()=>expect(divider()).not.toBeNull());
  expect(divider()).toHaveAttribute('aria-label','Views and queues');
  const outbox=sidebar().querySelector('[aria-label="Outbox"]')!;
  expect(divider()!.compareDocumentPosition(outbox)&Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
  expect(sidebar().querySelector('[data-folder-id="work"]')!.compareDocumentPosition(divider()!)&Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
 });
 it('is absent when there are no user folders',async()=>{
  await mount({},MAILBOXES);
  expect(sidebar().querySelector('[aria-label="Outbox"]')).not.toBeNull();
  expect(divider()).toBeNull();
 });
});

describe('the folder list is an ARIA tree',()=>{
 const tree=()=>screen.getByRole('tree',{name:'Mailboxes'});
 const item=(name:string)=>screen.getByRole('treeitem',{name});
 it('is a multiselectable tree of treeitems with level, expanded and selected state',async()=>{
  await mount();await waitFor(()=>expect(screen.getAllByRole('treeitem').length).toBeGreaterThan(3));
  expect(tree()).toHaveAttribute('aria-multiselectable','true');
  expect(item('Work')).toHaveAttribute('aria-level','2');
  expect(item('Quarter 3')).toHaveAttribute('aria-level','3');
  expect(item('Reports')).toHaveAttribute('aria-level','4');
  expect(item('Inbox')).toHaveAttribute('aria-level','1');
  expect(item('Work')).toHaveAttribute('aria-expanded','true');
  expect(item('Reports')).not.toHaveAttribute('aria-expanded');
  expect(item('Inbox')).toHaveAttribute('aria-selected','true');
  expect(item('Home')).toHaveAttribute('aria-selected','false');
  // The "User folders" node owns the group of custom folders.
  const header=screen.getByRole('treeitem',{name:'User folders'});
  expect(header).toHaveAttribute('aria-expanded','true');
  expect(document.getElementById(header.getAttribute('aria-controls')!)).toHaveAttribute('role','group');
 });
 it('is one Tab stop: the open folder, then wherever focus last was; the buttons inside are not tab stops',async()=>{
  await mount();await waitFor(()=>expect(screen.getAllByRole('treeitem').length).toBeGreaterThan(3));
  const stops=()=>screen.getAllByRole('treeitem').filter(el=>el.getAttribute('tabindex')==='0');
  expect(stops()).toHaveLength(1);expect(stops()[0]).toBe(item('Inbox'));
  item('Home').focus();
  await waitFor(()=>expect(stops()).toEqual([item('Home')]));
  expect(Array.from(tree().querySelectorAll('.sidebar__box,.sidebar__twisty')).every(el=>el.getAttribute('tabindex')==='-1')).toBe(true);
 });
 it('Home and End jump to the first and last folder; Enter and Space open the folder',async()=>{
  await mount();await waitFor(()=>expect(screen.getAllByRole('treeitem').length).toBeGreaterThan(3));
  const all=()=>screen.getAllByRole('treeitem');
  item('Work').focus();
  fireEvent.keyDown(item('Work'),{key:'End'});expect(document.activeElement).toBe(all().at(-1));
  fireEvent.keyDown(document.activeElement!,{key:'Home'});expect(document.activeElement).toBe(all()[0]);
  fireEvent.keyDown(item('Home'),{key:'Enter'});
  await waitFor(()=>expect(item('Home')).toHaveAttribute('aria-selected','true'));
 });
 it('typing jumps to the next folder whose name starts with the letters',async()=>{
  await mount();await waitFor(()=>expect(screen.getAllByRole('treeitem').length).toBeGreaterThan(3));
  item('Inbox').focus();
  fireEvent.keyDown(item('Inbox'),{key:'w'});expect(document.activeElement).toBe(item('Work'));
  fireEvent.keyDown(item('Work'),{key:'h'});
  // "wh" matches nothing: focus stays.
  expect(document.activeElement).toBe(item('Work'));
 });
 it('the "User folders" node collapses its group with Left and opens it with Right',async()=>{
  await mount();await waitFor(()=>expect(screen.getAllByRole('treeitem').length).toBeGreaterThan(3));
  const header=screen.getByRole('treeitem',{name:'User folders'});header.focus();
  fireEvent.keyDown(header,{key:'ArrowLeft'});
  await waitFor(()=>expect(header).toHaveAttribute('aria-expanded','false'));
  expect(screen.queryByRole('treeitem',{name:'Work'})).toBeNull();
  fireEvent.keyDown(header,{key:'ArrowRight'});
  await waitFor(()=>expect(header).toHaveAttribute('aria-expanded','true'));
  expect(item('Work')).toBeInTheDocument();
 });
});
