import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {render,screen,fireEvent,waitFor,within} from '@solidjs/testing-library';
import {ReaderToolbar,PRIORITY} from './ReaderToolbar.tsx';
import {inspectMessageLink} from '../viewers/messageLinks.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import type {Email} from '../api/jmap-types.ts';

const state=vi.hoisted(()=>({jmap:vi.fn(),open:vi.fn(),archive:vi.fn(),trash:vi.fn(),spam:vi.fn()}));
vi.mock('../api/transport.ts',()=>({createConfiguredClient:()=>({jmap:state.jmap})}));
vi.mock('../api/settings.ts',()=>({settings:()=>({'reader.actionStyle':{value:'icons'}})}));
vi.mock('../state/context.ts',()=>({useApp:()=>({accountId:()=> 'a',mailboxes:()=>[],openMessage:state.open,archiveMessage:state.archive,trashMessage:state.trash,pinMessage:vi.fn(),markRead:vi.fn(),flagMessage:vi.fn(),markSpam:state.spam,exportMessage:vi.fn()})}));

const email=(over:Partial<Email>={}):Email=>({id:'m1',threadId:'t1',subject:'Hello',from:[{name:'Ann',email:'ann@example.org'}],keywords:{$seen:true},mailboxIds:{inbox:true},...over}) as Email;
const thread=(ids:string[])=>({methodResponses:[['Thread/get',{accountId:'a',list:[{id:'t1',emailIds:ids}],notFound:[]},'thread']]});
const security={value:'full-sanitized' as const,floor:null,onChange:vi.fn()};
function toolbar(over:{email?:Email;links?:ReturnType<typeof inspectMessageLink>[]}={}){
 return render(()=><ReaderToolbar email={over.email??email()} onReply={vi.fn()} preparing={false} onPrint={vi.fn()} active links={(over.links??[]).filter(link=>link!==null)} linksTruncated={false} linkScope="a:m1" security={security}/>);
}

beforeEach(()=>activateAccountPreferences('a'));
afterEach(()=>{suspendAccountPreferences();vi.clearAllMocks();vi.unstubAllGlobals();vi.restoreAllMocks();});

describe('reader action row: conversation and links',()=>{
 it('shows View conversation only for a thread of more than one message',async()=>{
  state.jmap.mockResolvedValueOnce(thread(['m1']));
  toolbar();
  await waitFor(()=>expect(state.jmap).toHaveBeenCalledTimes(1));await Promise.resolve();
  expect(screen.queryByRole('button',{name:'View conversation'})).toBeNull();
 });
 it('opens the conversation dialog from the row and keeps its members',async()=>{
  state.jmap.mockResolvedValueOnce(thread(['m0','m1'])).mockResolvedValueOnce({methodResponses:[['Email/get',{accountId:'a',list:[{id:'m0',threadId:'t1',subject:'Earlier',mailboxIds:{}},{id:'m1',threadId:'t1',subject:'Hello',mailboxIds:{}}]},'members']]});
  toolbar();
  const trigger=await screen.findByRole('button',{name:'View conversation'});
  expect(trigger.closest('[role="toolbar"]')).not.toBeNull();expect(trigger).toHaveAttribute('aria-haspopup','dialog');expect(trigger).toHaveAttribute('title','View conversation');
  fireEvent.click(trigger);
  const dialog=await screen.findByRole('dialog',{name:'View conversation'});
  expect(within(dialog).queryByText(/Showing 0 of/)).toBeNull();expect(within(dialog).queryByRole('button',{name:'Load more messages'})).toBeNull();
  const earlier=await within(dialog).findByRole('button',{name:/Earlier/});expect(within(dialog).getByText('Showing 2 of 2 conversation messages.')).toBeVisible();fireEvent.click(earlier);expect(state.open).toHaveBeenCalledWith('m0');
 });
 it('hides Review links without links and shows the count badge with them',async()=>{
  state.jmap.mockResolvedValue(thread(['m1']));
  const first=toolbar();expect(screen.queryByRole('button',{name:/Review links/})).toBeNull();first.unmount();
  toolbar({links:[inspectMessageLink('https://a.example','A'),inspectMessageLink('https://b.example','B')]});
  const review=screen.getByRole('button',{name:'Review links (2)'});
  expect(review.querySelector('.reader-action__badge')?.textContent).toBe('2');expect(review.querySelector('.reader-action__badge')).toHaveAttribute('aria-hidden','true');
  fireEvent.click(review);expect(screen.getByRole('dialog',{name:'Message links'})).toBeVisible();
 });
});

describe('reader action row: overflow into More',()=>{
 /** Every action 40px, More 40px, gap 4px; the row is `rowWidth` wide. */
 function layout(rowWidth:number){
  let callback:(()=>void)|undefined;
  vi.stubGlobal('ResizeObserver',class{constructor(cb:()=>void){callback=cb;}observe(){}unobserve(){}disconnect(){}});
  vi.spyOn(HTMLElement.prototype,'getBoundingClientRect').mockImplementation(function(this:HTMLElement){const width=this.hidden?0:(this.dataset['readerItem']||this.dataset['testid']==='reader-more')?40:0;return {width,height:40,top:0,left:0,right:width,bottom:40,x:0,y:0,toJSON:()=>({})} as DOMRect;});
  vi.spyOn(HTMLElement.prototype,'clientWidth','get').mockImplementation(function(this:HTMLElement){return this.getAttribute('role')==='toolbar'?rowWidth:0;});
  const realStyle=window.getComputedStyle.bind(window);
  vi.spyOn(window,'getComputedStyle').mockImplementation(element=>{const style=realStyle(element);return (element as HTMLElement).getAttribute?.('role')==='toolbar'?{...style,columnGap:'4px',paddingLeft:'0px',paddingRight:'0px'} as CSSStyleDeclaration:style;});
  return ()=>callback?.();
 }
 it('keeps one row: the least important actions fold, in priority order, and run from More',async()=>{
  state.jmap.mockResolvedValue(thread(['m1']));
  const resize=layout(40*6+4*6);// room for 5 actions + More
  toolbar();resize();await Promise.resolve();resize();
  const row=screen.getByRole('toolbar');
  const shown=Array.from(row.querySelectorAll<HTMLElement>('[data-reader-item]')).filter(item=>!item.hidden).map(item=>item.dataset['readerItem']);
  expect(shown).toEqual(['reply','reply-all','forward','archive','delete']);
  const more=screen.getByRole('button',{name:'More actions'});expect(more).toHaveAttribute('aria-haspopup','menu');expect(more).toHaveAttribute('tabindex','0');
  fireEvent.click(more);
  const menu=screen.getByRole('menu',{name:'More actions'});
  const labels=Array.from(menu.querySelectorAll<HTMLElement>('[role^="menuitem"]')).map(item=>item.textContent);
  expect(labels.slice(0,3)).toEqual(['Pin','Mark unread','Flag']);
  expect(labels.filter(label=>label?.startsWith('View: '))).toEqual(['View: Full','View: No media','View: Plain text']);
  // Keyboard: the first item has focus, Escape returns it to More.
  expect(document.activeElement).toBe(Array.from(menu.querySelectorAll<HTMLElement>('[role^="menuitem"]'))[0]);
  fireEvent.keyDown(menu,{key:'Escape'});expect(screen.queryByRole('menu')).toBeNull();expect(document.activeElement).toBe(more);
  fireEvent.click(more);fireEvent.click(within(screen.getByRole('menu')).getByRole('menuitem',{name:'Spam'}));expect(state.spam).toHaveBeenCalledWith('m1');
 });
 it('folds by PRIORITY: Delete and Archive outlast the conversation, links and the rest',()=>{
  const order=Object.entries(PRIORITY).sort((a,b)=>a[1]-b[1]).map(([id])=>id);
  expect(order.slice(0,7)).toEqual(['reply','reply-all','forward','delete','archive','conversation','links']);
 });
 it('shows no More button when everything fits',async()=>{
  state.jmap.mockResolvedValue(thread(['m1']));
  const resize=layout(4000);toolbar();resize();await Promise.resolve();
  expect(screen.getByRole('toolbar').querySelectorAll('[data-reader-item][hidden]')).toHaveLength(0);
  expect(screen.getByTestId('reader-more')).toHaveAttribute('tabindex','-1');expect(screen.getByTestId('reader-more')).toHaveAttribute('aria-hidden','true');
 });
});
