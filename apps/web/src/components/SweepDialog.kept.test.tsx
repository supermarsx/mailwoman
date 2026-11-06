import {afterEach,describe,expect,it,vi} from 'vitest';
import {render,screen} from '@solidjs/testing-library';
import {SweepDialog} from './SweepDialog.tsx';
import {AppContext} from '../state/context.ts';
import type {AppState} from '../state/store.ts';
import {clearSettings,loadSettings,type Fetcher} from '../api/settings.ts';

const json=(body:unknown)=>new Response(JSON.stringify(body),{status:200,headers:{'content-type':'application/json'}});
afterEach(()=>{clearSettings();localStorage.clear();});

async function mount(keep:string[]){
 await loadSettings({refresh:true,fetcher:(async()=>json({settings:{'mail.list.keep':{value:keep,source:'user',locked:false}}})) as Fetcher});
 const executeSweep=vi.fn(async()=>undefined);
 const app={sweepPreview:()=>[{id:'1',subject:'a',receivedAt:'2026-11-01T00:00:00Z'}],executeSweep} as unknown as AppState;
 render(()=><AppContext.Provider value={app}><SweepDialog fromEmail="News@Acme.example" onClose={()=>undefined}/></AppContext.Provider>);
 return executeSweep;
}

describe('Clean up this sender and Always keep',()=>{
 it('is switched off for a kept sender, with the reason shown',async()=>{
  await mount(['sender:news@acme.example']);
  expect(screen.getByTestId('sweep-kept')).toBeInTheDocument();
  expect(screen.getByRole('button',{name:/Sweep/})).toBeDisabled();
 });
 it('runs for a sender that is not kept',async()=>{
  await mount([]);
  expect(screen.queryByTestId('sweep-kept')).toBeNull();
  expect(screen.getByRole('button',{name:/Sweep/})).not.toBeDisabled();
 });
});
