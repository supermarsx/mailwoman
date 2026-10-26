import {beforeEach,afterEach,it,expect,vi} from 'vitest';
import {render,fireEvent} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {useSettingsScroll} from './useSettingsScroll.ts';
import type {SettingsRoute} from './navigation.tsx';
let frames:Map<number,FrameRequestCallback>,next:number;
beforeEach(()=>{frames=new Map();next=0;vi.stubGlobal('requestAnimationFrame',(callback:FrameRequestCallback)=>{frames.set(++next,callback);return next;});vi.stubGlobal('cancelAnimationFrame',(id:number)=>frames.delete(id));activateAccountPreferences('account','msa1_first');});
afterEach(()=>{suspendAccountPreferences();vi.unstubAllGlobals();});
function flush(){const pending=[...frames.values()];frames.clear();for(const callback of pending)callback(0);}
function setup(){const [route,setRoute]=createSignal<SettingsRoute>({section:'appearance'});let node!:HTMLDivElement;function Host(){useSettingsScroll(route,()=>node);return <div ref={node}><input/></div>;}render(()=><Host/>);flush();return {node,setRoute,scroll:(top:number)=>{node.scrollTop=top;fireEvent.scroll(node);}};}
it('restores independent section and editor positions without changing routes',()=>{const {node,setRoute,scroll}=setup();scroll(420);setRoute({section:'folders'});flush();expect(node.scrollTop).toBe(0);scroll(180);setRoute({section:'appearance'});flush();expect(node.scrollTop).toBe(420);setRoute({section:'appearance',editor:{kind:'edit',id:'one'}});flush();expect(node.scrollTop).toBe(0);setRoute({section:'folders'});flush();expect(node.scrollTop).toBe(180);});
it('cancels obsolete route frames and respects newer user scroll intent',()=>{const {node,setRoute,scroll}=setup();scroll(420);setRoute({section:'folders'});setRoute({section:'time'});flush();expect(node.scrollTop).toBe(0);setRoute({section:'appearance'});fireEvent.wheel(node);scroll(75);flush();expect(node.scrollTop).toBe(75);});
it('clears private route positions and scheduled restoration on same-ID realm changes',()=>{const {node,setRoute,scroll}=setup();scroll(420);setRoute({section:'folders'});flush();setRoute({section:'appearance'});suspendAccountPreferences();activateAccountPreferences('account','msa1_second');flush();expect(node.scrollTop).toBe(0);setRoute({section:'folders'});flush();setRoute({section:'appearance'});flush();expect(node.scrollTop).toBe(0);});
