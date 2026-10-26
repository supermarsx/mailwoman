import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {render,screen,waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {avatarPresentation,PersonAvatar} from './PersonAvatar.tsx';
const state=vi.hoisted(()=>({mode:'initials',fetch:vi.fn(),owner:{accountId:'one',generation:1}}));
vi.mock('../api/settings.ts',()=>({settings:()=>({'contacts.avatarMode':{value:state.mode}})}));
vi.mock('../api/http.ts',()=>({apiFetch:(...args:unknown[])=>state.fetch(...args)}));
vi.mock('../api/accountPreferenceLifecycle.ts',()=>({observePreferenceAccount:(listener:(value:unknown)=>void)=>{listener(state.owner);return ()=>{};},preferenceAccount:()=>state.owner,preferenceAccountIsCurrent:(owner:unknown)=>owner===state.owner}));
afterEach(()=>vi.unstubAllGlobals());
beforeEach(()=>{state.mode='initials';state.fetch.mockReset();});
it('generates stable local initials and makes no photo request by default',()=>{
 expect(avatarPresentation('Ana Maria','ana@example.org').initials).toBe('AM');expect(avatarPresentation('Ana Maria','ana@example.org')).toEqual(avatarPresentation('Ana Maria','ana@example.org'));
 expect(avatarPresentation('\u202eAna','ana@example.org').initials).toBe('A');render(()=><PersonAvatar name="Ana Maria" address="ana@example.org"/>);expect(screen.getByText('AM')).toBeTruthy();expect(state.fetch).not.toHaveBeenCalled();
});
it('hides pictures without fetching when disabled',()=>{state.mode='off';const view=render(()=><PersonAvatar name="Ana" address="ana@example.org"/>);expect(view.container.textContent).toBe('');expect(state.fetch).not.toHaveBeenCalled();});
it('uses a same-origin body request and falls back without retry on refusal',async()=>{state.mode='contacts';state.fetch.mockResolvedValue(new Response(null,{status:204}));render(()=><PersonAvatar name="Ana" address="ana@example.org"/>);await waitFor(()=>expect(state.fetch).toHaveBeenCalledTimes(1));expect(state.fetch.mock.calls[0]?.[0]).toBe('/api/account/contact-photo');expect(JSON.parse(state.fetch.mock.calls[0]?.[1].body)).toEqual({address:'ana@example.org'});expect(screen.getByText('A')).toBeTruthy();});
it('discards a late photo when the component address changes',async()=>{
 state.mode='contacts';let resolve!:(value:Response)=>void;state.fetch.mockImplementationOnce(()=>new Promise<Response>(done=>resolve=done)).mockResolvedValue(new Response(null,{status:204}));
 const create=vi.fn(()=> 'blob:photo');vi.stubGlobal('URL',class extends URL {static override createObjectURL=create;static override revokeObjectURL=vi.fn();});const [address,setAddress]=createSignal('ana@example.org');render(()=><PersonAvatar name="Ana" address={address()}/>);setAddress('other@example.org');resolve(new Response(new Uint8Array([1,2]),{headers:{'content-type':'image/png'}}));await waitFor(()=>expect(state.fetch).toHaveBeenCalledTimes(2));expect(create).not.toHaveBeenCalled();
});
