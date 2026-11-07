import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {render,screen,waitFor} from '@solidjs/testing-library';
import {createSignal} from 'solid-js';
import {avatarPresentation,PersonAvatar} from './PersonAvatar.tsx';
import {AVATAR_COLORS} from './senderIcons/palette.ts';
const state=vi.hoisted(()=>({mode:'initials',icons:true,brands:true as boolean,requireAuth:undefined as string|undefined,fetch:vi.fn(),owner:{accountId:'one',generation:1}}));
vi.mock('../api/settings.ts',()=>({settings:()=>({'contacts.avatarMode':{value:state.mode},'contacts.senderIcons':{value:state.icons},'mail.brandAvatars':{value:state.brands},'mail.brandAvatars.requireAuth':{value:state.requireAuth}})}));
vi.mock('../api/http.ts',()=>({apiFetch:(...args:unknown[])=>state.fetch(...args)}));
vi.mock('../api/accountPreferenceLifecycle.ts',()=>({observePreferenceAccount:(listener:(value:unknown)=>void)=>{listener(state.owner);return ()=>{};},preferenceAccount:()=>state.owner,preferenceAccountIsCurrent:(owner:unknown)=>owner===state.owner}));
afterEach(()=>vi.unstubAllGlobals());
beforeEach(()=>{state.mode='initials';state.icons=true;state.brands=true;state.requireAuth=undefined;state.fetch.mockReset();});
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
const proven={dkim:{result:'pass',domain:'github.com'},dmarc:{result:'pass',aligned:true}};
const row=(address:string,auth?:unknown)=>render(()=><PersonAvatar name="Sender" address={address} messageSender auth={auth as never}/>).container;
it('a message row shows a brand icon for the sender domain when the verdict is unknown, passed or merely unaligned',()=>{
 for(const auth of [null,undefined,proven,{...proven,dmarc:{result:'none',aligned:false}},{...proven,dmarc:{result:'pass',aligned:false}},{...proven,dkim:{result:'none',domain:null}},{...proven,dkim:{result:'pass',domain:'evil.example'}}]){
  expect(row('notifications@github.com',auth).querySelector('[data-sender-icon="github"]')).not.toBeNull();
 }
});
it('a message row with a known authentication failure shows initials and an unverified tooltip',()=>{
 for(const auth of [{...proven,dmarc:{result:'fail',aligned:false}},{...proven,dmarc:{result:'fail',aligned:true}},{dkim:{result:'fail',domain:'github.com'},dmarc:{result:'none',aligned:false}}]){
  const view=row('notifications@github.com',auth);
  expect(view.querySelector('[data-sender-icon]')).toBeNull();expect(view).toHaveTextContent('S');
  expect(view.querySelector('[data-unverified="true"]')).not.toBeNull();expect(view.querySelector('.person-avatar')?.getAttribute('data-tooltip')).toMatch(/nverified|não verificado/i);
 }
 // DKIM failed but an aligned DMARC pass (SPF) rescued the message: not a failure.
 expect(row('notifications@github.com',{dkim:{result:'fail',domain:'github.com'},dmarc:{result:'pass',aligned:true}}).querySelector('[data-sender-icon="github"]')).not.toBeNull();
});
it('shows the icon for subdomains of the brand at any depth, and not for lookalikes',()=>{
 for(const address of ['a@e.amazon.com','a@mail.zalando.com','a@em.paypal.com','a@notifications.github.com','a@news.vinted.com','a@service.fnac.pt','a@a.b.c.paypal.com','a@EM.PayPal.COM'])expect(row(address).querySelector('[data-sender-icon]'),address).not.toBeNull();
 for(const address of ['a@paypal.com.evil.io','a@evilpaypal.com','a@pay-pal.com','a@notgithub.com'])expect(row(address).querySelector('[data-sender-icon]'),address).toBeNull();
});
it('a registry brand outside the bundled catalogue gets its mark in a message row, never for a person on a public mailbox provider',()=>{
 expect(row('news@mail.ryanair.com').querySelector('[data-sender-icon="ryanair"]')).not.toBeNull();
 expect(row('friend@gmail.com').querySelector('[data-sender-icon]')).toBeNull();
 expect(render(()=><PersonAvatar name="R" address="news@mail.ryanair.com"/>).container.querySelector('[data-sender-icon]')).toBeNull();
});
it('requireAuth never ignores a failure, always needs a pass, known-failure is the default',()=>{
 const failed={...proven,dmarc:{result:'fail',aligned:false}};
 state.requireAuth='never';expect(row('a@github.com',failed).querySelector('[data-sender-icon="github"]')).not.toBeNull();
 state.requireAuth='always';
 expect(row('a@github.com',proven).querySelector('[data-sender-icon="github"]')).not.toBeNull();
 for(const auth of [null,failed,{...proven,dkim:{result:'pass',domain:'evil.example'}}])expect(row('a@github.com',auth).querySelector('[data-sender-icon]')).toBeNull();
 expect(row('a@bsky.app',{dkim:{result:'pass',domain:'bsky.social'},dmarc:{result:'pass',aligned:true}}).querySelector('[data-sender-icon]')).not.toBeNull();
 expect(row('a@bsky.app',null).querySelector('[data-sender-icon]')).toBeNull();
 state.requireAuth='nonsense';expect(row('a@github.com',failed).querySelector('[data-sender-icon]')).toBeNull();expect(row('a@github.com',null).querySelector('[data-sender-icon="github"]')).not.toBeNull();
});
it('a message row needs no verdict for a generic public-body glyph, and a contact needs none at all',()=>{
 expect(render(()=><PersonAvatar name="Finanças" address="aviso@portaldasfinancas.gov.pt" messageSender/>).container.querySelector('[data-sender-icon="government"]')).not.toBeNull();
 expect(render(()=><PersonAvatar name="GitHub" address="notifications@github.com"/>).container.querySelector('[data-sender-icon="github"]')).not.toBeNull();
});
it('mail.brandAvatars off hides registry marks in rows but keeps the catalogue icons',()=>{
 state.brands=false;
 expect(row('news@mail.ryanair.com').querySelector('[data-sender-icon]')).toBeNull();
 expect(row('a@github.com').querySelector('[data-sender-icon="github"]')).not.toBeNull();
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
