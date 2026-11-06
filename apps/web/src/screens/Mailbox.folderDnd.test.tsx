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
import {clearSelectedFolders} from '../components/folderSelection.ts';
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

describe('Pinned group',()=>{
 const group=()=>document.querySelector<HTMLElement>('[data-testid="sidebar-pinned-group"]');
 const pinnedNames=()=>Array.from(group()?.querySelectorAll('.sidebar__box-name')??[]).map(node=>node.textContent);
 afterEach(()=>{setAccountUiPreferences({folderPinned:[]});});
 it('lists pinned folders above the tree while they keep their tree position',async()=>{
  await mount();
  expect(group()).toBeNull();
  setAccountUiPreferences({folderPinned:['misc','home']});
  await waitFor(()=>expect(pinnedNames()).toEqual(['Misc','Home']));
  expect(group()).toHaveAttribute('aria-label','Pinned');
  expect(sidebar().querySelectorAll('[data-folder-id="home"]')).toHaveLength(1);
  expect(group()!.querySelector('[data-folder-id]')).toBeNull();
 });
 it('hides a pinned folder without subfolders from the tree when the administrator asks, keeping parents',async()=>{
  await mount({'folders.pinned.hideFromTree':true});
  setAccountUiPreferences({folderPinned:['home','work']});
  await waitFor(()=>expect(pinnedNames()).toEqual(['Home','Work']));
  expect(sidebar().querySelector('[data-folder-id="home"]')).toBeNull();
  expect(sidebar().querySelector('[data-folder-id="work"]')).not.toBeNull();
 });
 it('reorders pins by dragging within the group and unpins by dropping outside it',async()=>{
  await mount();
  setAccountUiPreferences({folderPinned:['misc','home']});
  await waitFor(()=>expect(pinnedNames()).toEqual(['Misc','Home']));
  const pinRow=(id:string)=>group()!.querySelector<HTMLElement>(`[data-pin-id="${id}"]`)!;
  const types=['application/x-mailwoman-pin'];
  const dt=()=>({setData:()=>undefined,types});
  const target=pinRow('home');target.getBoundingClientRect=()=>({top:0,left:0,height:40,width:240,bottom:40,right:240,x:0,y:0,toJSON:()=>({})});
  fireEvent(pinRow('home'),createEvent.dragStart(pinRow('home'),{dataTransfer:dt()}));
  await new Promise(resolve=>setTimeout(resolve,5));
  const over=createEvent.dragOver(pinRow('misc'),{dataTransfer:dt()});Object.defineProperty(over,'clientY',{value:1});
  pinRow('misc').getBoundingClientRect=target.getBoundingClientRect;
  fireEvent(pinRow('misc'),over);
  fireEvent(pinRow('misc'),createEvent.drop(pinRow('misc'),{dataTransfer:dt()}));
  await waitFor(()=>expect(accountUiPreferences().folderPinned).toEqual(['home','misc']));
  // Drag one out: dropping on the navigation outside the group unpins it.
  fireEvent(pinRow('misc'),createEvent.dragStart(pinRow('misc'),{dataTransfer:dt()}));
  await new Promise(resolve=>setTimeout(resolve,5));
  const nav=sidebar().querySelector('.sidebar__nav')!;
  fireEvent(nav,createEvent.dragOver(nav,{dataTransfer:dt()}));
  fireEvent(nav,createEvent.drop(nav,{dataTransfer:dt()}));
  await waitFor(()=>expect(accountUiPreferences().folderPinned).toEqual(['home']));
 });
 it('shows a drag handle on sidebar rows that can be dragged, and honours the setting',async()=>{
  await mount();
  await waitFor(()=>expect(names()).toContain('Work'));
  expect(rowOf('work').querySelector('[data-testid="folder-grab-handle"]')).not.toBeNull();
  expect(rowOf('work')).toHaveAttribute('data-handles','hover');
  setAccountUiPreferences({folderGrabHandles:'always'});
  await waitFor(()=>expect(rowOf('work')).toHaveAttribute('data-handles','always'));
  setAccountUiPreferences({folderGrabHandles:'hover'});
 });
});

describe('selecting folders in the sidebar',{timeout:15000},()=>{
 const btn=(name:string)=>screen.getByRole('button',{name});
 const picked=()=>Array.from(sidebar().querySelectorAll('[data-folder-id][data-selected="true"]')).map(node=>node.getAttribute('data-folder-id')).sort();
 afterEach(()=>{setAccountUiPreferences({folderCollapsed:[]});clearSelectedFolders();});
 it('Ctrl+click toggles, Shift+click selects the range in tree order, and neither opens the folder',async()=>{
  await mount();
  await waitFor(()=>expect(names()).toContain('Home'));
  fireEvent.click(btn('Work'),{ctrlKey:true});
  expect(picked()).toEqual(['work']);
  fireEvent.click(btn('Misc'),{shiftKey:true});
  await waitFor(()=>expect(picked()).toEqual(['home','misc','q3','work'].sort()));
  fireEvent.click(btn('Home'),{shiftKey:true});
  await waitFor(()=>expect(picked()).toEqual(['home','q3','work'].sort()));
  fireEvent.click(btn('Q3'.replace('Q3','Quarter 3')),{ctrlKey:true});
  await waitFor(()=>expect(picked()).toEqual(['home','work']));
  expect(screen.getByTestId('folder-selection-bar')).toHaveTextContent('2');
 });
 it('range selection skips subfolders hidden by a collapsed folder',async()=>{
  await mount();
  await waitFor(()=>expect(names()).toContain('Quarter 3'));
  setAccountUiPreferences({folderCollapsed:['work']});
  await waitFor(()=>expect(names()).not.toContain('Quarter 3'));
  fireEvent.click(btn('Work'),{ctrlKey:true});
  fireEvent.click(btn('Misc'),{shiftKey:true});
  await waitFor(()=>expect(picked()).toEqual(['home','misc','work']));
 });
 it('Shift+Arrow extends, Ctrl+A takes the user folders, Escape clears',async()=>{
  await mount();
  await waitFor(()=>expect(names()).toContain('Home'));
  fireEvent.click(btn('Home'),{ctrlKey:true});
  fireEvent.keyDown(btn('Home'),{key:'ArrowDown',shiftKey:true});
  await waitFor(()=>expect(picked()).toEqual(['home','misc']));
  fireEvent.keyDown(btn('Home'),{key:'a',ctrlKey:true});
  await waitFor(()=>expect(picked()).toEqual(['home','misc','q3','work']));
  expect(btn('Inbox').closest('[data-folder-id]')?.getAttribute('data-selected')??null).toBeNull();
  fireEvent.keyDown(btn('Home'),{key:'Escape'});
  await waitFor(()=>expect(picked()).toEqual([]));
  expect(screen.queryByTestId('folder-selection-bar')).toBeNull();
 });
 it('the selection bar pins and stars the whole set, and a bulk delete names folders and messages',async()=>{
  await mount();
  await waitFor(()=>expect(names()).toContain('Home'));
  fireEvent.click(btn('Home'),{ctrlKey:true});fireEvent.click(btn('Misc'),{ctrlKey:true});
  fireEvent.click(screen.getByTestId('bar-pin'));
  await waitFor(()=>expect(accountUiPreferences().folderPinned).toEqual(expect.arrayContaining(['home','misc'])));
  fireEvent.click(screen.getByTestId('bar-star'));
  await waitFor(()=>expect(accountUiPreferences().folderStarred).toEqual(expect.arrayContaining(['home','misc'])));
  // Without folder-management ability the destructive actions are off, with a reason.
  expect(screen.getByTestId('bar-delete')).toBeDisabled();
  setAccountUiPreferences({folderPinned:[],folderStarred:[]});
 });
});
