import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,render,waitFor} from '@solidjs/testing-library';
import {BrandLogo} from './BrandLogo.tsx';
import {PersonAvatar} from '../components/PersonAvatar.tsx';
import {SenderBrandMark} from './SenderBrandMark.tsx';
import {MailboxAvatar} from '../components/accounts/MailboxAvatar.tsx';
import {clearSettings} from '../api/settings.ts';
import {suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import { loadBrandRegistry } from './registry.ts';
await loadBrandRegistry();

vi.mock('../components/senderIcons/match.ts',async original=>{
 const actual=await original<typeof import('../components/senderIcons/match.ts')>();
 return {...actual,senderDomainIcon:(address:string)=>address==='mark@inline.test'?{id:'inline-vector',name:'Inline vector',domains:['inline.test'],paths:[],asset:'data:image/svg+xml,%3Csvg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"/%3E'}:actual.senderDomainIcon(address)};
});
afterEach(()=>{cleanup();clearSettings();suspendAccountPreferences();});

it('keeps each provider mark in its requested square before lazy artwork loads',async()=>{
 for(const size of [16,20,24,32] as const){
  const {container,unmount}=render(()=><BrandLogo brand="gmail" size={size}/>);
  const mark=container.querySelector('svg')!;
  expect(mark.getAttribute('width')).toBe(String(size));
  expect(mark.getAttribute('height')).toBe(String(size));
  expect(mark.getAttribute('preserveAspectRatio')).toBe('xMidYMid meet');
  await waitFor(()=>expect(mark.querySelector('path')).not.toBeNull());
  expect(mark.getAttribute('width')).toBe(String(size));
  unmount();
 }
});
it('distinguishes scalable wordmarks from native small raster artwork',()=>{
 const raster=render(()=><PersonAvatar address="mail@cgd.pt"/>).container.querySelector('img')!;
 expect(raster.classList.contains('person-avatar__brand')).toBe(true);
 expect(raster.getAttribute('data-vendor-scalable')).toBeNull();
 expect(raster.getAttribute('src')).toMatch(/cgd/);
 const vector=render(()=><PersonAvatar address="mail@tranquilidade.pt"/>).container.querySelector('img')!;
 expect(vector.getAttribute('data-vendor-scalable')).toBe('true');
 expect(vector.getAttribute('src')).toMatch(/generali-tranquilidade/);
 const inline=render(()=><PersonAvatar address="mark@inline.test"/>).container.querySelector('img')!;
 expect(inline.getAttribute('data-vendor-scalable')).toBe('true');
});
it('uses contained vector viewports for list marks and keeps reader artwork in its own slot',()=>{
 const list=render(()=><PersonAvatar address="mail@github.com"/>).container.querySelector('svg')!;
 expect(list.getAttribute('viewBox')).toBe('0 0 24 24');
 expect(list.getAttribute('preserveAspectRatio')).toBe('xMidYMid meet');
 const reader=render(()=><SenderBrandMark address="mail@viacard.pt" auth={null}/>).container;
 expect(reader.querySelector('.person-avatar[data-sender-icon] img.person-avatar__brand')).not.toBeNull();
});
it('preserves saved account glyph rendering without classifying it as vendor artwork',()=>{
 const {container}=render(()=><MailboxAvatar label="Work" address="me@example.test" size="lg" avatar={{kind:'icon',icon:'star',bg:'#123456',fg:'#ffffff'}}/>);
 const avatar=container.querySelector('.person-avatar')!;
 expect(avatar.getAttribute('data-size')).toBe('lg');
 expect(avatar.getAttribute('data-avatar-icon')).toBe('star');
 expect(avatar.hasAttribute('data-sender-icon')).toBe(false);
 expect(avatar.querySelector('svg')).not.toBeNull();
 expect(avatar.querySelector('.person-avatar__brand')).toBeNull();
});
