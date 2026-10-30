// t68: in a detached message window, an action that changes the message in
// place (Flag, Read, Pin, zoom, Show images) runs there; only actions that open
// a dialog or menu (which render in the main window) or leave the message move
// the pane back first.
import {afterEach,expect,it,vi} from 'vitest';
import {render,waitFor} from '@solidjs/testing-library';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {DetachedWorkspacePane,type DetachedPaneHandle} from './DetachedWorkspacePane.tsx';
import {ReaderToolbar} from './ReaderToolbar.tsx';
import {RemoteContentBar} from './RemoteContentBar.tsx';
import type {Email} from '../api/jmap-types.ts';

const state=vi.hoisted(()=>({flag:vi.fn(async()=>{}),read:vi.fn(async()=>{}),pin:vi.fn(async()=>{}),archive:vi.fn(async()=>{})}));
vi.mock('../api/transport.ts',()=>({createConfiguredClient:()=>({jmap:vi.fn(async()=>({methodResponses:[['Thread/get',{accountId:'a',list:[{id:'t1',emailIds:['m1']}],notFound:[]},'thread']]}))})}));
vi.mock('../api/settings.ts',async original=>({
 ...await original<typeof import('../api/settings.ts')>(),
 settings:()=>({'reader.actionStyle':{value:'icons'}}),
 loadSettings:vi.fn(async()=>undefined),
 settingsLoadStatus:()=> 'ready',
 writeSetting:vi.fn(async(_key:string,value:unknown)=>({value,source:'user',locked:false})),
}));
vi.mock('../state/context.ts',()=>({useApp:()=>({accountId:()=>'a',mailboxes:()=>[],openMessage:vi.fn(),archiveMessage:state.archive,trashMessage:vi.fn(),pinMessage:state.pin,markRead:state.read,flagMessage:state.flag,markSpam:vi.fn(),exportMessage:vi.fn()})}));

afterEach(()=>{suspendAccountPreferences();localStorage.clear();vi.restoreAllMocks();state.flag.mockClear();state.read.mockClear();state.pin.mockClear();state.archive.mockClear();});
function popup(){const doc=document.implementation.createHTMLDocument('popup');doc.documentElement.setAttribute('data-mailwoman-workspace-popup','1');Object.defineProperty(doc,'readyState',{value:'complete',configurable:true});const events=new EventTarget();const value={document:doc,location:{href:new URL('/workspace-popup.html',location.href).href},closed:false,focus:vi.fn(),close:vi.fn(()=>{value.closed=true;}),addEventListener:events.addEventListener.bind(events),removeEventListener:events.removeEventListener.bind(events),dispatchEvent:events.dispatchEvent.bind(events)};vi.spyOn(window,'open').mockReturnValue(value as unknown as Window);return value;}
/** The popup document has no window of its own here; dispatch directly. */
const click=(element:Element):void=>{element.dispatchEvent(new MouseEvent('click',{bubbles:true,cancelable:true}));};
const email={id:'m1',threadId:'t1',subject:'Hello',from:[{name:'Ann',email:'ann@example.org'}],keywords:{$seen:true},mailboxIds:{inbox:true}} as unknown as Email;

async function detached(){
 vi.spyOn(window,'focus').mockImplementation(()=>{});activateAccountPreferences('a','msa1_abcdefghijklmnop');const child=popup();let handle:DetachedPaneHandle|undefined;const grant=vi.fn(async()=>{});
 render(()=><DetachedWorkspacePane title={()=>'Hello'} register={value=>{handle=value;}}>{()=><>
  <ReaderToolbar email={email} onReply={vi.fn()} preparing={false} onPrint={vi.fn()} active links={[]} linksTruncated={false} linkScope="a:m1" security={{value:'full-sanitized',floor:null,onChange:vi.fn()}}/>
  <RemoteContentBar emailId="m1" sender="ann@example.org" report={{blockedHosts:['img.example.net'],blockedCount:2,trackerCount:0}} activeGrant={null} mode="ask" onGrant={grant}/>
 </>}</DetachedWorkspacePane>);
 handle!.detach();
 await waitFor(()=>expect(child.document.querySelector('[data-testid="reader-flag"]')).not.toBeNull());
 return {child,handle:handle!,grant,doc:child.document};
}

it('keeps a detached message window open for Flag, Read, Pin and zoom',async()=>{
 const {child,handle,doc}=await detached();
 click(doc.querySelector('[data-testid="reader-flag"]')!);
 expect(state.flag).toHaveBeenCalledWith('m1',true);
 click(doc.querySelector('[data-testid="reader-toggle-read"]')!);
 expect(state.read).toHaveBeenCalledWith('m1',false);
 click(doc.querySelector('[data-reader-item="pin"] button')!);
 expect(state.pin).toHaveBeenCalledWith('m1',true);
 click(doc.querySelector('[data-testid="reader-zoom-in"]')!);
 expect(child.closed).toBe(false);expect(child.close).not.toHaveBeenCalled();expect(handle.isDetached()).toBe(true);
 expect(doc.querySelector('[data-testid="reader-flag"]')).not.toBeNull();
});

it('keeps it open for Show images, and still moves back for an action that leaves the message',async()=>{
 const {child,handle,grant,doc}=await detached();
 const show=[...doc.querySelectorAll<HTMLButtonElement>('[data-testid="remote-content-bar"] button')].find(button=>button.hasAttribute('data-detached-local'));
 expect(show).toBeDefined();click(show!);
 await waitFor(()=>expect(grant).toHaveBeenCalled());
 expect(child.closed).toBe(false);expect(handle.isDetached()).toBe(true);
 click(doc.querySelector('[data-reader-item="archive"] button')!);
 await waitFor(()=>expect(child.closed).toBe(true));expect(handle.isDetached()).toBe(false);
 await waitFor(()=>expect(state.archive).toHaveBeenCalledWith('m1'));
});
