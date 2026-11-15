import {afterEach,expect,it} from 'vitest';
import {waitFor} from '@solidjs/testing-library';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {toast,toasts} from '../components/toast/toast.ts';
import {createKeymap} from './dispatcher.ts';
import {SHORTCUT_TOAST_KEY} from './activationToast.ts';
let dispose:()=>void=()=>{};
afterEach(()=>{dispose();clearSettings();toast.clear();document.body.innerHTML='';});
const configure=async(enabled:boolean)=>loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{[SHORTCUT_TOAST_KEY]:{value:enabled,source:'global',locked:true}}}))});
const setup=()=>{const keymap=createKeymap({target:document,bindings:()=>new Map([['e','archive'],['mod+n','compose']])});dispose=keymap.dispose;keymap.register('archive',()=>true);keymap.register('compose',()=>true);return keymap;};
it('is silent by default and acknowledges a handled key only when enabled',async()=>{
 const keymap=setup();keymap.dispatch(new KeyboardEvent('keydown',{key:'e',cancelable:true}));expect(toasts()).toEqual([]);
 await configure(true);keymap.dispatch(new KeyboardEvent('keydown',{key:'e',cancelable:true}));await waitFor(()=>expect(toasts()).toHaveLength(1));
 expect(toasts()[0]).toMatchObject({message:'Shortcut: Archive (E)',duration:2000,testId:'shortcut-activation-toast'});
 keymap.dispatch(new KeyboardEvent('keydown',{key:'n',ctrlKey:true,cancelable:true}));await waitFor(()=>expect(toasts()[0]?.message).toBe('Shortcut: Compose (Ctrl+N)'));expect(toasts()).toHaveLength(1);
 await configure(false);toast.clear();keymap.dispatch(new KeyboardEvent('keydown',{key:'e',cancelable:true}));await Promise.resolve();expect(toasts()).toEqual([]);
});
it('does not acknowledge programmatic actions, typing, repeats, or declined shortcuts',async()=>{
 await configure(true);const keymap=createKeymap({target:document,bindings:()=>new Map([['e','archive'],['mod+n','compose']])});dispose=keymap.dispose;keymap.register('archive',()=>true);expect(keymap.run('archive')).toBe(true);
 keymap.dispatch(new KeyboardEvent('keydown',{key:'e',repeat:true,cancelable:true}));keymap.register('compose',()=>false);
 keymap.dispatch(new KeyboardEvent('keydown',{key:'n',ctrlKey:true,cancelable:true}));
 const input=document.createElement('input');document.body.append(input);input.dispatchEvent(new KeyboardEvent('keydown',{key:'e',bubbles:true,cancelable:true}));
 await Promise.resolve();expect(toasts()).toEqual([]);
});
