// t38 §2.3: a torn-off window names its mailbox account in its title.
import {afterEach,expect,it,vi} from 'vitest';
import {render,waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {stripIsolates} from '../i18n/index.ts';
import {DetachedWorkspacePane,type DetachedPaneHandle} from './DetachedWorkspacePane.tsx';

vi.mock('../i18n/index.ts',async original=>{
  const real=await original<typeof import('../i18n/index.ts')>();
  return {...real,loadCatalog:async()=>undefined,t:(key:string,args?:Record<string,string>)=>key==='mail-popup-title'?`${args?.title} — ${args?.address}`:key};
});
afterEach(()=>{suspendAccountPreferences();vi.restoreAllMocks();});

function popup(){const doc=document.implementation.createHTMLDocument('popup');doc.documentElement.setAttribute('data-mailwoman-workspace-popup','1');Object.defineProperty(doc,'readyState',{value:'complete',configurable:true});const events=new EventTarget();const value={document:doc,location:{href:new URL('/workspace-popup.html',location.href).href},closed:false,focus:vi.fn(),close:vi.fn(()=>{value.closed=true;}),addEventListener:events.addEventListener.bind(events),removeEventListener:events.removeEventListener.bind(events),dispatchEvent:events.dispatchEvent.bind(events)};vi.spyOn(window,'open').mockReturnValue(value as unknown as Window);return value;}

it('puts the active address in the popup title and its header label, and follows renames',async()=>{
  activateAccountPreferences('same','msa1_abcdefghijklmnop');const child=popup();let handle:DetachedPaneHandle|undefined;
  const [title,setTitle]=createSignal('Draft');
  render(()=><DetachedWorkspacePane title={title} account={()=>'me@example.org'} register={value=>{handle=value;}}>{()=><input aria-label="Draft"/>}</DetachedWorkspacePane>);
  handle!.detach();
  await waitFor(()=>expect(handle!.isDetached()).toBe(true));
  expect(stripIsolates(child.document.title)).toBe('Draft — me@example.org');
  expect(stripIsolates(child.document.querySelector('.detached-workspace__header span')?.textContent??'')).toBe('Draft — me@example.org');
  setTitle('Quarterly report');
  await waitFor(()=>expect(stripIsolates(child.document.title)).toBe('Quarterly report — me@example.org'));
});

it('keeps the plain pane title without an account',async()=>{
  activateAccountPreferences('same','msa1_abcdefghijklmnop');const child=popup();let handle:DetachedPaneHandle|undefined;
  render(()=><DetachedWorkspacePane title={()=>'Draft'} register={value=>{handle=value;}}>{()=><input aria-label="Draft"/>}</DetachedWorkspacePane>);
  handle!.detach();
  await waitFor(()=>expect(handle!.isDetached()).toBe(true));
  expect(child.document.title).toBe('Draft');
});
