import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {render,screen,fireEvent,waitFor,within} from '@solidjs/testing-library';
import {ReaderToolbar,PRIORITY} from './ReaderToolbar.tsx';
import {READER_ACTIONS,readerActionKey} from './readerActionPlacement.ts';
import type {ContextAction} from './ContextMenu.tsx';
import type {Mailbox} from '../api/jmap-types.ts';
import {inspectMessageLink} from '../viewers/messageLinks.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {saveDraft} from './compose/drafts-store.ts';
import type {Email} from '../api/jmap-types.ts';
import {fetchMailHeaders} from '../api/mailHeaders.ts';

const state=vi.hoisted(()=>({jmap:vi.fn(),open:vi.fn(),archive:vi.fn(),trash:vi.fn(),finish:vi.fn().mockResolvedValue(undefined),spam:vi.fn(),search:vi.fn(),moveFocus:vi.fn(),focusEnabled:false,focusValue:undefined as 'focused'|'other'|undefined,boxes:[] as Mailbox[],extras:undefined as ContextAction[]|undefined,settings:{} as Record<string,{value:unknown;locked?:boolean;source?:string}>,bumpSettings:()=>{}}));
vi.mock('../realtime/context.ts',()=>({useRealtime:()=>({subTabs:{tabs:()=>[],activate:vi.fn()}})}));
vi.mock('../api/transport.ts',()=>({createConfiguredClient:()=>({jmap:state.jmap})}));
vi.mock('../api/mailHeaders.ts',async original=>({...await original<typeof import('../api/mailHeaders.ts')>(),fetchMailHeaders:vi.fn()}));
vi.mock('../api/settings.ts',async()=>{const {createSignal}=await vi.importActual<typeof import('solid-js')>('solid-js');const [tick,bump]=createSignal(0);state.bumpSettings=()=>bump(value=>value+1);return {settings:()=>{tick();return {'reader.actionStyle':{value:'icons'},...state.settings};}};});
vi.mock('../modules/contacts/saveFromMessage.rows.ts',async original=>{const actual=await original<typeof import('../modules/contacts/saveFromMessage.rows.ts')>();return {...actual,saveRowsFor:(...args:Parameters<typeof actual.saveRowsFor>)=>state.extras??actual.saveRowsFor(...args)};});
vi.mock('../state/context.ts',()=>({useApp:()=>({identities:()=>[],contacts:()=>[],accountId:()=> 'a',supportsFromDomainSearch:()=>true,searchMessageField:state.search,mailboxes:()=>state.boxes,focusedInbox:()=>state.focusEnabled,senderFocus:()=>state.focusValue,alwaysMoveSender:state.moveFocus,showToast:vi.fn(),openMessage:state.open,archiveMessage:state.archive,trashMessage:state.trash,completeFollowUp:state.finish,pinMessage:vi.fn(),markRead:vi.fn(),flagMessage:vi.fn(),markSpam:state.spam,exportMessage:vi.fn()})}));

const email=(over:Partial<Email>={}):Email=>({id:'m1',threadId:'t1',subject:'Hello',from:[{name:'Ann',email:'ann@example.org'}],keywords:{$seen:true},mailboxIds:{inbox:true},...over}) as Email;
const thread=(ids:string[])=>({methodResponses:[['Thread/get',{accountId:'a',list:[{id:'t1',emailIds:ids}],notFound:[]},'thread']]});
const security={value:'full-sanitized' as const,floor:null,onChange:vi.fn()};
function toolbar(over:{onEditDraft?:(id:string)=>void;onOpenReplyDraft?:(id:string)=>void;email?:Email;links?:ReturnType<typeof inspectMessageLink>[];headersUrl?:string}={}){
 return render(()=><ReaderToolbar email={over.email??email()} {...(over.onOpenReplyDraft?{onOpenReplyDraft:over.onOpenReplyDraft}:{})} {...(over.onEditDraft?{onEditDraft:over.onEditDraft}:{})} onReply={vi.fn()} preparing={false} onPrint={vi.fn()} active links={(over.links??[]).filter(link=>link!==null)} linksTruncated={false} linkScope="a:m1" {...(over.headersUrl?{headersUrl:over.headersUrl}:{})} headersScope="account:message" security={security}/>);
}

describe('reply draft action',()=>{
 it('shows Open draft beside Forward only while a saved reply exists, and opens it',async()=>{
  const open=vi.fn();toolbar({onOpenReplyDraft:open});
  expect(screen.queryByTestId('reader-open-draft')).toBeNull();
  const authority='test-storage-authority-a';activateAccountPreferences('a',authority);
  saveDraft({id:'rd1',accountId:'a',storageAuthority:authority,to:'a@example.org',subject:'Re: Hello',bodyHtml:'<p>x</p>',bodyText:'x',savedAt:1,source:{emailId:'m1',keyword:'$answered'}});
  const button=await screen.findByTestId('reader-open-draft');
  expect(button.getAttribute('aria-label')??button.textContent).toMatch(/Open draft/);
  fireEvent.click(button);expect(open).toHaveBeenCalledWith('rd1');
 });
});

describe('draft message action',()=>{
 it('offers Edit draft first, in place of Reply and Forward, when the open message is a draft',async()=>{
  const edit=vi.fn();toolbar({email:email({keywords:{$draft:true,$seen:true}}),onEditDraft:edit});
  const button=await screen.findByTestId('reader-edit-draft');
  expect(button.getAttribute('aria-label')??button.textContent).toMatch(/Edit draft/);
  expect(screen.queryByTestId('reader-reply')).toBeNull();expect(screen.queryByTestId('reader-forward')).toBeNull();
  fireEvent.click(button);expect(edit).toHaveBeenCalledWith('m1');
 });
 it('keeps Reply and offers no Edit draft for an ordinary message',()=>{
  toolbar({onEditDraft:vi.fn()});
  expect(screen.queryByTestId('reader-edit-draft')).toBeNull();expect(screen.getByTestId('reader-reply')).toBeTruthy();
 });
});

beforeEach(()=>{activateAccountPreferences('a');state.settings={};state.extras=undefined;state.focusEnabled=false;state.focusValue=undefined;state.boxes=[];state.bumpSettings();});
const configure=(values:Record<string,unknown>)=>{state.settings=Object.fromEntries(Object.entries(values).map(([key,value])=>[key,{value}]));state.bumpSettings();};
afterEach(()=>{suspendAccountPreferences();vi.clearAllMocks();vi.unstubAllGlobals();vi.restoreAllMocks();});

describe('reader action row: conversation and links',()=>{
 it('opens original message headers beside the view selector on demand and returns focus on close',async()=>{
  state.jmap.mockResolvedValue(thread(['m1']));vi.mocked(fetchMailHeaders).mockResolvedValue('Subject: Hello\r\nReceived: folded\r\n\tcontinued');
  toolbar({headersUrl:'/mail/raw'});
  const trigger=screen.getByRole('button',{name:'Raw message headers'});
  expect(document.querySelector('[data-reader-item="security"]')?.nextElementSibling).toBe(trigger.closest('[data-reader-item="headers"]'));
  expect(trigger).toHaveAttribute('aria-haspopup','dialog');expect(fetchMailHeaders).not.toHaveBeenCalled();
  fireEvent.click(trigger);
  const dialog=await screen.findByRole('dialog',{name:'Raw message headers'});
  await waitFor(()=>expect(within(dialog).getByLabelText('Raw message headers')).toHaveTextContent('Subject: Hello'));
  expect(fetchMailHeaders).toHaveBeenCalledWith('/mail/raw',expect.any(AbortSignal));
  fireEvent.keyDown(dialog,{key:'Escape'});await waitFor(()=>expect(screen.queryByRole('dialog')).toBeNull());
  expect(trigger).toHaveFocus();
 });
 it('moves related search into the actions row and preserves domain search',async()=>{
  state.jmap.mockResolvedValue(thread(['m1']));toolbar();
  const trigger=await screen.findByRole('button',{name:'Search related messages'});
  expect(trigger.closest('[role="toolbar"]')).not.toBeNull();fireEvent.click(trigger);
  fireEvent.click(await screen.findByRole('menuitem',{name:'Search messages from this domain'}));
  expect(state.search).toHaveBeenCalledWith('from-domain','example.org');
 });
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
 it('folds by PRIORITY: Delete, Archive and Move outlast the conversation, links and the rest',()=>{
  const order=Object.entries(PRIORITY).sort((a,b)=>a[1]-b[1]).map(([id])=>id);
  expect(order.slice(0,10)).toEqual(['edit-draft','reply','reply-all','forward','open-draft','delete','archive','move','conversation','links'].slice(0,10));
 });
 it('folds nothing when everything fits, and More stays because it holds the message-only actions',async()=>{
  state.jmap.mockResolvedValue(thread(['m1']));
  const resize=layout(4000);toolbar();resize();await Promise.resolve();
  expect(screen.getByRole('toolbar').querySelectorAll('[data-reader-item][data-reader-placement="toolbar"][hidden]')).toHaveLength(0);
  expect(screen.getByTestId('reader-more')).not.toHaveAttribute('aria-hidden','true');
 });
 it('keeps the header dialog reachable from More on narrow readers',async()=>{
  state.jmap.mockResolvedValue(thread(['m1']));vi.mocked(fetchMailHeaders).mockResolvedValue('Subject: Narrow');
  const resize=layout(40*6+4*6);toolbar({headersUrl:'/mail/raw'});resize();await Promise.resolve();resize();
  expect(screen.queryByRole('button',{name:'Raw message headers'})).toBeNull();
  const more=screen.getByRole('button',{name:'More actions'});fireEvent.click(more);
  fireEvent.click(within(screen.getByRole('menu',{name:'More actions'})).getByRole('menuitem',{name:'Raw message headers'}));
  const dialog=await screen.findByRole('dialog',{name:'Raw message headers'});
  fireEvent.keyDown(dialog,{key:'Escape'});await waitFor(()=>expect(screen.queryByRole('dialog')).toBeNull());expect(more).toHaveFocus();
 });
 it('sends forced-menu actions directly to More and hides hidden actions on both surfaces',async()=>{
  state.jmap.mockResolvedValue(thread(['m1']));configure({'reader.actions.archive':'menu','reader.actions.delete':'hidden'});
  const resize=layout(4000);toolbar();resize();await Promise.resolve();
  expect(screen.queryByRole('button',{name:'Archive'})).toBeNull();expect(screen.queryByRole('button',{name:'Delete'})).toBeNull();
  fireEvent.click(screen.getByRole('button',{name:'More actions'}));const menu=screen.getByRole('menu',{name:'More actions'});
  expect(within(menu).queryByRole('menuitem',{name:'Delete'})).toBeNull();fireEvent.click(within(menu).getByRole('menuitem',{name:'Archive'}));expect(state.archive).toHaveBeenCalledWith('m1');
 });
 it('moves a live action through menu, hidden and toolbar without replacing its button',async()=>{
  state.jmap.mockResolvedValue(thread(['m1']));const resize=layout(4000);toolbar();resize();await Promise.resolve();
  const original=screen.getByTestId('reader-reply');
  configure({'reader.actions.reply':'menu'});await Promise.resolve();resize();expect(screen.queryByRole('button',{name:'Reply'})).toBeNull();
  expect(document.querySelector('[data-testid=reader-reply]')).toBe(original);
  fireEvent.click(screen.getByRole('button',{name:'More actions'}));expect(within(screen.getByRole('menu')).getByRole('menuitem',{name:'Reply'})).toBeVisible();
  configure({'reader.actions.reply':'hidden'});await Promise.resolve();expect(screen.queryByRole('menu')).toBeNull();
  fireEvent.click(screen.getByRole('button',{name:'More actions'}));expect(within(screen.getByRole('menu')).queryByRole('menuitem',{name:'Reply'})).toBeNull();
  configure({});await Promise.resolve();resize();expect(screen.getByRole('button',{name:'Reply'})).toBe(original);
 });
 it('promotes a menu-only contacts action as a working submenu without flattening its children',async()=>{
  const run=vi.fn();state.extras=[{id:'save-contacts',label:'Save contacts',icon:'more',run:()=>{},children:[{id:'choose-contact',label:'Pick a contact',icon:'more',run}]}];
  state.jmap.mockResolvedValue(thread(['m1']));configure({'reader.actions.save-contacts':'toolbar'});
  const resize=layout(4000);toolbar();resize();await Promise.resolve();
  const promoted=screen.getByRole('button',{name:'Save contacts'});expect(promoted).toHaveAttribute('aria-haspopup','menu');fireEvent.click(promoted);
  fireEvent.click(screen.getByRole('menuitem',{name:'Pick a contact'}));expect(run).toHaveBeenCalledOnce();
  fireEvent.click(screen.getByRole('button',{name:'More actions'}));expect(within(screen.getByRole('menu')).queryByRole('menuitem',{name:'Save contacts'})).toBeNull();
 });
 it('promotes actual sender Focused actions with their original identity, checked state and execution',async()=>{
  state.focusEnabled=true;state.focusValue='focused';state.jmap.mockResolvedValue(thread(['m1']));configure({'reader.actions.focus-focused':'toolbar'});
  const resize=layout(4000);toolbar();resize();await Promise.resolve();
  const promoted=screen.getByTestId('reader-focus-focused');expect(promoted).toHaveAttribute('aria-pressed','true');fireEvent.click(promoted);
  expect(state.moveFocus).toHaveBeenCalledWith('ann@example.org','focused');
  fireEvent.click(screen.getByRole('button',{name:'More actions'}));expect(within(screen.getByRole('menu')).queryByRole('menuitemcheckbox',{name:/Always move to Focused/})).toBeNull();
 });
 it('reserves More for a forced-menu action and measures only toolbar candidates',async()=>{
  state.jmap.mockResolvedValue(thread(['m1']));const values=Object.fromEntries(READER_ACTIONS.map(item=>[readerActionKey(item.id),'hidden']));
  configure({...values,'reader.actions.reply':'toolbar','reader.actions.archive':'menu'});
  const resize=layout(83);toolbar();resize();await Promise.resolve();resize();
  // Reply40 + gap4 + More40 needs84: the toolbar preference must overflow.
  expect(screen.queryByRole('button',{name:'Reply'})).toBeNull();
  const measured=vi.mocked(HTMLElement.prototype.getBoundingClientRect).mock.contexts.map(item=>(item as HTMLElement).dataset?.['readerItem']).filter(Boolean);
  expect(new Set(measured)).toEqual(new Set(['reply']));
  const more=screen.getByRole('button',{name:'More actions'});expect(more).not.toHaveAttribute('aria-hidden','true');fireEvent.click(more);
  const menu=screen.getByRole('menu');expect(within(menu).getByRole('menuitem',{name:'Reply'})).toBeVisible();expect(within(menu).getByRole('menuitem',{name:'Archive'})).toBeVisible();
  configure({...values,'reader.actions.archive':'menu'});await Promise.resolve();resize();
  expect(screen.getByRole('button',{name:'More actions'})).toBeVisible();
  fireEvent.click(screen.getByRole('button',{name:'More actions'}));
  expect(within(screen.getByRole('menu')).getAllByRole('menuitem').map(item=>item.textContent)).toEqual(['Archive']);
 });
 it('keeps feature-gated actions unavailable even when their placement requests the toolbar',async()=>{
  state.jmap.mockResolvedValue(thread(['m1']));configure({...Object.fromEntries(READER_ACTIONS.map(item=>[readerActionKey(item.id),'toolbar'])),'mail.allowReply':false,'mail.allowForward':false,'mail.allowPrint':false,'mail.allowDownload':false,'mail.showSpam':false,'properties.message.enabled':false,'contacts.saveSender.enabled':false,'contacts.saveFromMessage.enabled':false});
  const resize=layout(4000);toolbar();resize();await Promise.resolve();
  for(const id of ['reply','reply-all','forward','export','print','spam','properties','save-sender','save-contacts'])expect(document.querySelector(`[data-reader-item="${id}"]`)).toBeNull();
 });
 it('preserves unknown future menu actions in More rather than dropping them',async()=>{
  const run=vi.fn();state.extras=[{id:'future-action',label:'Future action',icon:'more',run}];state.jmap.mockResolvedValue(thread(['m1']));
  configure({'reader.actions.future-action':'toolbar'});const resize=layout(4000);toolbar();resize();await Promise.resolve();
  expect(screen.queryByRole('button',{name:'Future action'})).toBeNull();fireEvent.click(screen.getByRole('button',{name:'More actions'}));
  fireEvent.click(screen.getByRole('menuitem',{name:'Future action'}));expect(run).toHaveBeenCalledOnce();
 });

 it('retains disabled state when promoting a menu-only action',async()=>{
  const run=vi.fn();state.extras=[{id:'save-sender',label:'Save sender',icon:'more',disabled:true,title:'No sender available',run}];
  state.jmap.mockResolvedValue(thread(['m1']));configure({'reader.actions.save-sender':'toolbar'});
  const resize=layout(4000);toolbar();resize();await Promise.resolve();
  const promoted=screen.getByRole('button',{name:'Save sender'});expect(promoted).toBeDisabled();expect(promoted).toHaveAttribute('title','No sender available');
  fireEvent.click(promoted);expect(run).not.toHaveBeenCalled();
 });

 it('returns focus to More after closing headers forced into the menu on a wide reader',async()=>{
  state.jmap.mockResolvedValue(thread(['m1']));vi.mocked(fetchMailHeaders).mockResolvedValue('Subject: Forced menu');
  configure({'reader.actions.headers':'menu'});const resize=layout(4000);toolbar({headersUrl:'/mail/raw'});resize();await Promise.resolve();
  const more=screen.getByRole('button',{name:'More actions'});fireEvent.click(more);
  fireEvent.click(within(screen.getByRole('menu')).getByRole('menuitem',{name:'Raw message headers'}));
  const dialog=await screen.findByRole('dialog',{name:'Raw message headers'});fireEvent.keyDown(dialog,{key:'Escape'});
  await waitFor(()=>expect(screen.queryByRole('dialog')).toBeNull());expect(more).toHaveFocus();
 });

});


describe('Finished reader action',()=>{
 it('offers Finished only for an active follow-up and invokes the undoable state action',async()=>{
  state.jmap.mockResolvedValue(thread(['m1']));toolbar({email:email({followUpAt:'2026-11-09T12:00:00Z'})});
  fireEvent.click(screen.getByTestId('reader-more'));const action=document.querySelector<HTMLButtonElement>('[data-action-id="finish-followup"]');expect(action).not.toBeNull();fireEvent.click(action!);
  expect(state.finish).toHaveBeenCalledWith('m1');
 });
});
