import {afterEach,expect,it,vi} from 'vitest';
import {render,screen,waitFor} from '@solidjs/testing-library';
import {onCleanup} from 'solid-js';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {DetachedWorkspacePane,type DetachedPaneHandle} from './DetachedWorkspacePane.tsx';
afterEach(()=>{suspendAccountPreferences();vi.restoreAllMocks();});
function popup(){const doc=document.implementation.createHTMLDocument('popup');doc.documentElement.setAttribute('data-mailwoman-workspace-popup','1');Object.defineProperty(doc,'readyState',{value:'complete',configurable:true});const events=new EventTarget();const value={document:doc,location:{href:new URL('/workspace-popup.html',location.href).href},closed:false,focus:vi.fn(),close:vi.fn(()=>{value.closed=true;}),addEventListener:events.addEventListener.bind(events),removeEventListener:events.removeEventListener.bind(events),dispatchEvent:events.dispatchEvent.bind(events)};vi.spyOn(window,'open').mockReturnValue(value as unknown as Window);return value;}
it('moves the same live editor back without remounting and closes on authority replacement',async()=>{
 activateAccountPreferences('same','msa1_abcdefghijklmnop');const child=popup();let handle:DetachedPaneHandle|undefined;let mounts=0,cleanups=0;
 const Editor=()=>{mounts++;onCleanup(()=>cleanups++);return <input aria-label="Draft" value="Unsaved"/>;};
 render(()=><DetachedWorkspacePane title={()=>'Draft'} register={value=>{handle=value;}}>{()=><Editor/>}</DetachedWorkspacePane>);
 const original=screen.getByLabelText('Draft');handle!.detach();await waitFor(()=>expect(child.document.querySelector('input')).toBe(original));
 const leaving=new Event('beforeunload',{cancelable:true});window.dispatchEvent(leaving);expect(leaving.defaultPrevented).toBe(true);
 handle!.reattach();await waitFor(()=>expect(screen.getByLabelText('Draft')).toBe(original));expect(mounts).toBe(1);expect(cleanups).toBe(0);
 const other=popup();handle!.detach();await waitFor(()=>expect(other.document.querySelector('input')).toBe(original));activateAccountPreferences('same','msa1_changedabcdefgh');await waitFor(()=>expect(other.closed).toBe(true));
});
it('leaves the original pane intact when the browser blocks the popup',()=>{
 activateAccountPreferences('a');vi.spyOn(window,'open').mockReturnValue(null);let handle:DetachedPaneHandle|undefined;
 render(()=><DetachedWorkspacePane title={()=>'Draft'} register={value=>{handle=value;}}>{()=><input aria-label="Draft"/>}</DetachedWorkspacePane>);
 const original=screen.getByLabelText('Draft');handle!.detach();expect(screen.getByLabelText('Draft')).toBe(original);expect(handle!.isDetached()).toBe(false);
});
it('waits for navigation to finish before moving the live draft',async()=>{
 activateAccountPreferences('same','msa1_abcdefghijklmnop');const child=popup();let handle:DetachedPaneHandle|undefined;
 Object.defineProperty(child.document,'readyState',{value:'loading',configurable:true});
 render(()=><DetachedWorkspacePane title={()=>'Draft'} register={value=>{handle=value;}}>{()=><input aria-label="Draft" value="Unsaved"/>}</DetachedWorkspacePane>);
 const original=screen.getByLabelText('Draft');handle!.detach();
 expect(screen.getByLabelText('Draft')).toBe(original);expect(child.document.querySelector('input')).toBeNull();
 Object.defineProperty(child.document,'readyState',{value:'complete',configurable:true});
 child.dispatchEvent(new Event('load'));
 await waitFor(()=>expect(child.document.querySelector('input')).toBe(original));
 expect(handle!.isDetached()).toBe(true);
});
it('rejects a 404 document at the expected URL and keeps unsaved content in the main window',()=>{
 activateAccountPreferences('same','msa1_abcdefghijklmnop');const child=popup();let handle:DetachedPaneHandle|undefined;
 child.document.documentElement.removeAttribute('data-mailwoman-workspace-popup');child.document.body.textContent='Error 404';
 render(()=><DetachedWorkspacePane title={()=>'Draft'} register={value=>{handle=value;}}>{()=><input aria-label="Draft" value="Unsaved"/>}</DetachedWorkspacePane>);
 const original=screen.getByLabelText('Draft');handle!.detach();
 expect(child.closed).toBe(true);expect(handle!.isDetached()).toBe(false);
 expect(screen.getByLabelText('Draft')).toBe(original);expect(original).toHaveValue('Unsaved');
 expect(child.document.querySelector('.detached-workspace__header')).toBeNull();
});
