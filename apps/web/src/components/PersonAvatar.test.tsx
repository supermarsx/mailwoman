import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {render,screen,waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {avatarPresentation,PersonAvatar} from './PersonAvatar.tsx';
import {AVATAR_COLORS} from './senderIcons/palette.ts';
const state=vi.hoisted(()=>({mode:'initials',icons:true,fetch:vi.fn(),owner:{accountId:'one',generation:1}}));
vi.mock('../api/settings.ts',()=>({settings:()=>({'contacts.avatarMode':{value:state.mode},'contacts.senderIcons':{value:state.icons}})}));
vi.mock('../api/http.ts',()=>({apiFetch:(...args:unknown[])=>state.fetch(...args)}));
vi.mock('../api/accountPreferenceLifecycle.ts',()=>({observePreferenceAccount:(listener:(value:unknown)=>void)=>{listener(state.owner);return ()=>{};},preferenceAccount:()=>state.owner,preferenceAccountIsCurrent:(owner:unknown)=>owner===state.owner}));
afterEach(()=>vi.unstubAllGlobals());
beforeEach(()=>{state.mode='initials';state.icons=true;state.fetch.mockReset();});
it('uses 192 stable colours with readable white initials and no inline style',()=>{
 expect(new Set(AVATAR_COLORS).size).toBe(192);
 for(const hex of AVATAR_COLORS){
  const channels=[1,3,5].map(start=>parseInt(hex.slice(start,start+2),16)/255).map(v=>v<=.04045?v/12.92:((v+.055)/1.055)**2.4);
  expect(1.05/(.2126*channels[0]!+.7152*channels[1]!+.0722*channels[2]!+.05)).toBeGreaterThanOrEqual(4.5);
 }
 expect(new Set(Array.from({length:1000},(_,n)=>avatarPresentation('',`sender${n}@example.org`).color)).size).toBeGreaterThan(150);
 expect(avatarPresentation('','ANA@EXAMPLE.ORG').palette).toBe(avatarPresentation('','ana@example.org').palette);
 const view=render(()=><PersonAvatar name="Ana" address="ana@example.org"/>);
 expect(view.container.querySelector('.person-avatar')).not.toHaveAttribute('style');
});
it('uses a bundled SVG for recognised senders without any photo requests',()=>{
 const view=render(()=><PersonAvatar name="GitHub" address="notifications@github.com"/>);
 expect(view.container.querySelector('[data-sender-icon="github"] svg path')).not.toBeNull();
 expect(state.fetch).not.toHaveBeenCalled();
});
it('allows user or admin policy to disable sender icons',()=>{
 state.icons=false;
 const view=render(()=><PersonAvatar name="GitHub" address="notifications@github.com"/>);
 expect(view.container.querySelector('svg')).toBeNull();expect(view.container).toHaveTextContent('G');
});
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

it('renders bundled official artwork or an honest compact monogram without external requests',()=>{
 const view=render(()=><><PersonAvatar address="mail@openrouter.ai"/><PersonAvatar address="mail@olx.pt"/><PersonAvatar address="mail@credibom.pt"/></>);
 for(const id of ['openrouter','olx']){
  const tile=view.container.querySelector(`[data-sender-icon="${id}"]`)!;const img=tile.querySelector('img')!;
  expect(img).toHaveClass('person-avatar__brand');expect(img.src.startsWith('data:image/')||new URL(img.src,location.href).origin===location.origin).toBe(true);expect(tile).not.toHaveAttribute('style');
 }
 expect(view.container.querySelector('[data-sender-icon="credibom"] .person-avatar__monogram')).toHaveTextContent('CB');
 expect(state.fetch).not.toHaveBeenCalled();
});
