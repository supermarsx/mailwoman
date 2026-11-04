import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {fireEvent,screen} from '@solidjs/testing-library';
import {MessageList} from './MessageList.tsx';
import {mkEmail,renderWithApp} from './appHarness.tsx';
import {clearSettings,loadSettings,type Fetcher} from '../api/settings.ts';

const load=(value?:boolean)=>loadSettings({refresh:true,fetcher:(async()=>new Response(JSON.stringify({settings:value===undefined?{}:{'mail.showGoToTop':{value,source:'user',locked:false}}}),{headers:{'content-type':'application/json'}})) as Fetcher});
const emails=()=>Array.from({length:40},(_,i)=>mkEmail(`m${i}`,{subject:`Message ${i}`}));
const mount=async()=>{const {app,result}=renderWithApp(()=><MessageList/>,{emails:emails()});await app.login({username:'me@example.org',password:'p'});await screen.findByText('Message 0');return result;};

beforeEach(()=>localStorage.clear());
afterEach(()=>{clearSettings();vi.unstubAllGlobals();});

describe('MessageList Go to top button (mail.showGoToTop)',()=>{
 it('is hidden by default',async()=>{
  await load();await mount();
  expect(screen.queryByRole('button',{name:'Go to top'})).toBeNull();
 });
 it('is shown, focusable and named when the setting is on',async()=>{
  await load(true);await mount();
  const button=await screen.findByRole('button',{name:'Go to top'});
  expect(button).toHaveAttribute('title','Go to top');
  expect(button.tabIndex).toBeGreaterThanOrEqual(0);
 });
 it('scrolls the list back to the first message, smoothly by default',async()=>{
  await load(true);const result=await mount();
  const scroller=result.container.querySelector('.list__scroll') as HTMLElement;
  scroller.scrollTop=900;
  const scrollTo=vi.fn();scroller.scrollTo=scrollTo as unknown as typeof scroller.scrollTo;
  fireEvent.click(await screen.findByRole('button',{name:'Go to top'}));
  expect(scrollTo).toHaveBeenCalledWith({top:0,behavior:'smooth'});
 });
 it('scrolls instantly when the user prefers reduced motion',async()=>{
  vi.stubGlobal('matchMedia',(query:string)=>({matches:query.includes('prefers-reduced-motion'),media:query,addEventListener:()=>undefined,removeEventListener:()=>undefined,addListener:()=>undefined,removeListener:()=>undefined}));
  await load(true);const result=await mount();
  const scroller=result.container.querySelector('.list__scroll') as HTMLElement;
  const scrollTo=vi.fn();scroller.scrollTo=scrollTo as unknown as typeof scroller.scrollTo;
  fireEvent.click(await screen.findByRole('button',{name:'Go to top'}));
  expect(scrollTo).toHaveBeenCalledWith({top:0,behavior:'auto'});
 });
});
