import {afterEach,beforeEach,describe,expect,it} from 'vitest';
import {cleanup,createEvent,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {AppContext} from '../state/context.ts';
import {createAppState} from '../state/store.ts';
import {makeClient,mkEmail,MAILBOXES} from '../components/appHarness.tsx';
import {MailboxScreen} from './Mailbox.tsx';
import {RealtimeContext} from '../realtime/context.ts';
import {createRealtimeController} from '../realtime/controller.ts';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {accountUiPreferences,setAccountUiPreferences} from '../api/accountPreferences.ts';
import {WORKSPACE_STORAGE_KEY} from '../realtime/workspaceStorage.ts';
import {FOLDER_DRAG_TYPE} from '../components/useFolderDnd.ts';
import type {Mailbox} from '../api/jmap-types.ts';

// Sidebar drag and drop of folders (reorder, nest, locks) and the starred marker.
const TREE:Mailbox[]=[...MAILBOXES,
 {id:'work',name:'Work',parentId:null,role:null,sortOrder:10,totalEmails:5,unreadEmails:1},
 {id:'q3',name:'Quarter 3',parentId:'work',role:null,sortOrder:10,totalEmails:2,unreadEmails:4},
 {id:'home',name:'Home',parentId:null,role:null,sortOrder:11,totalEmails:0,unreadEmails:0},
 {id:'misc',name:'Misc',parentId:null,role:null,sortOrder:12,totalEmails:0,unreadEmails:0},
];
const apply=(values:Record<string,unknown>)=>loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:Object.fromEntries(Object.entries(values).map(([key,value])=>[key,{value,source:'user',locked:false}]))}),{headers:{'content-type':'application/json'}})});
beforeEach(async()=>{suspendAccountPreferences();clearSettings();localStorage.removeItem(WORKSPACE_STORAGE_KEY);history.replaceState(null,'','#/mail');await new Promise(resolve=>setTimeout(resolve,0));});
afterEach(()=>{cleanup();clearSettings();setAccountUiPreferences({folderStarred:[],folderOrder:[],folderParent:{},folderNestMode:'ask'});});
async function mount(values:Record<string,unknown>={}){
 const app=createAppState(makeClient({emails:[mkEmail('a')],mailboxes:TREE}));const realtime=createRealtimeController();
 await apply(values);await app.login({username:'me@example.org',password:'p'});
 render(()=><AppContext.Provider value={app}><RealtimeContext.Provider value={realtime}><MailboxScreen/></RealtimeContext.Provider></AppContext.Provider>);
 await screen.findByText('Subject a',{selector:'.list__subject'});
}
const sidebar=()=>document.querySelector('aside.sidebar') as HTMLElement;
const rowOf=(id:string)=>sidebar().querySelector<HTMLElement>(`[data-folder-id="${id}"]`)!.closest<HTMLElement>('[style*="--folder-depth"]')!;
const names=()=>Array.from(sidebar().querySelectorAll('[data-folder-region] .sidebar__box:not([data-folder-disclosure]) .sidebar__box-name')).map(node=>node.textContent);
const transfer=()=>({setData:()=>undefined,types:[FOLDER_DRAG_TYPE]});
/** Drag `source` and release it over `target`; jsdom has no layout, so the row rectangle and pointer are set by hand. */
function drag(source:string,target:string,zone:'before'|'inside'|'after'):void{
 const row=rowOf(target);row.getBoundingClientRect=()=>({top:0,left:0,height:40,width:240,bottom:40,right:240,x:0,y:0,toJSON:()=>({})});
 fireEvent(rowOf(source),createEvent.dragStart(rowOf(source),{dataTransfer:transfer()}));
 const over=createEvent.dragOver(row,{dataTransfer:transfer()});
 Object.defineProperty(over,'clientY',{value:zone==='before'?2:zone==='inside'?20:38});Object.defineProperty(over,'clientX',{value:10});
 fireEvent(row,over);
 fireEvent(row,createEvent.drop(row,{dataTransfer:transfer()}));
}

describe('starred folders',()=>{
 it('show a decorative star and "Starred" in the description, only on starred rows',async()=>{
  await mount();
  await waitFor(()=>expect(names()).toContain('Work'));
  setAccountUiPreferences({folderStarred:['work']});
  await waitFor(()=>expect(sidebar().querySelectorAll('[data-testid="folder-star"]')).toHaveLength(1));
  const stars=sidebar().querySelectorAll('[data-testid="folder-star"]');
  expect(stars).toHaveLength(1);expect(stars[0]).toHaveAttribute('aria-hidden','true');
  expect(screen.getByRole('button',{name:'Work'})).toHaveAttribute('aria-description',expect.stringContaining('Starred'));
  expect(screen.getByRole('button',{name:'Home'}).getAttribute('aria-description')??'').not.toContain('Starred');
  expect(screen.getByRole('button',{name:'Work'}).getAttribute('title')).toContain('Starred');
 });
 it('can be shown alone, with the folders that contain them',async()=>{
  await mount();
  setAccountUiPreferences({folderStarred:['q3']});
  const toggle=await screen.findByRole('button',{name:/Show starred only/});
  expect(toggle).toHaveAttribute('aria-pressed','false');
  fireEvent.click(toggle);
  await waitFor(()=>expect(names()).not.toContain('Home'));
  expect(names()).toEqual(expect.arrayContaining(['Work','Quarter 3']));expect(toggle).toHaveAttribute('aria-pressed','true');
  fireEvent.click(toggle);await waitFor(()=>expect(names()).toContain('Home'));
 });
 it('toggle from the folder menu stores the star without touching pins',async()=>{
  await mount();
  fireEvent.contextMenu(screen.getByRole('button',{name:'Home'}));
  fireEvent.click(await screen.findByRole('menuitem',{name:'Star folder'}));
  expect(accountUiPreferences().folderStarred).toEqual(['home']);expect(accountUiPreferences().folderPinned??[]).toEqual([]);
 });
});

describe('dragging folders in the sidebar',()=>{
 it('reorders among siblings in the preferences only',async()=>{
  await mount();await waitFor(()=>expect(names()).toContain('Misc'));
  drag('misc','home','before');
  await waitFor(()=>{const list=names();expect(list.indexOf('Misc')).toBeLessThan(list.indexOf('Home'));});
  expect(accountUiPreferences().folderParent??{}).toEqual({});
 });
 it('nests with a view-only arrangement when the choice is "only arrange in this app"',async()=>{
  await mount();await waitFor(()=>expect(names()).toContain('Misc'));
  setAccountUiPreferences({folderNestMode:'local'});
  drag('misc','home','inside');
  await waitFor(()=>expect(accountUiPreferences().folderParent).toEqual({misc:'home'}));
  const depth=(id:string)=>rowOf(id).style.getPropertyValue('--folder-depth');
  await waitFor(()=>expect(depth('misc')).toBe('1'));
 });
 it('keeps a nest as an arrangement, without asking, when the account cannot change the server',async()=>{
  await mount();await waitFor(()=>expect(names()).toContain('Misc'));
  drag('misc','home','inside');
  await waitFor(()=>expect(accountUiPreferences().folderParent).toEqual({misc:'home'}));
  expect(screen.queryByRole('alertdialog')).toBeNull();
 });
 it('is off for every folder when the administrator fixes the order, with a lock hint',async()=>{
  await mount({'folders.order.locked':true});
  await waitFor(()=>expect(names()).toContain('Home'));
  expect(rowOf('home').getAttribute('draggable')).toBe('false');
  expect(rowOf('home')).toHaveAttribute('title','The folder order is fixed by your administrator.');
  drag('misc','home','before');
  expect(names().indexOf('Misc')).toBeGreaterThan(names().indexOf('Home'));
 });
 it('does not drag system folders',async()=>{
  await mount();await waitFor(()=>expect(names()).toContain('Home'));
  expect(sidebar().querySelector<HTMLElement>('[data-folder-id="inbox"]')!.closest('[style*="--folder-depth"]')!.getAttribute('draggable')).toBe('false');
 });
 it('moves with Alt+ArrowUp from the keyboard',async()=>{
  await mount();await waitFor(()=>expect(names()).toContain('Misc'));
  const misc=screen.getByRole('button',{name:'Misc'});misc.focus();
  fireEvent.keyDown(misc,{key:'ArrowUp',altKey:true});
  await waitFor(()=>{const list=names();expect(list.indexOf('Misc')).toBeLessThan(list.indexOf('Home'));});
  expect(screen.getByTestId('folder-dnd-status')).toHaveTextContent(/Misc.*moved up/);
 });
});
