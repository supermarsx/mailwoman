import {afterEach,describe,expect,it} from 'vitest';
import {render,screen,waitFor} from '@solidjs/testing-library';
import {clearSettings,loadSettings,type Fetcher} from '../../api/settings.ts';
import {MailFeaturePreferences} from './MailFeaturePreferences.tsx';

const json=(body:unknown):Response=>new Response(JSON.stringify(body),{status:200,headers:{'content-type':'application/json'}});
async function load(locked:boolean):Promise<void>{
 await loadSettings({refresh:true,fetcher:(async()=>json({settings:{'mail.rowActionsMode':{value:'hover',source:locked?'global':'default',locked}}})) as Fetcher});
}
afterEach(()=>clearSettings());

describe('row actions mode setting',()=>{
 it('offers the four modes and is editable by the user when not locked',async()=>{
  await load(false);
  render(()=><MailFeaturePreferences/>);
  const select=await screen.findByLabelText('Message list quick actions') as HTMLSelectElement;
  expect([...select.options].map(o=>o.value)).toEqual(['open','hover','click','off']);
  expect([...select.options].map(o=>o.text)).toEqual(['Always open','Show on hover','Show on click','Off']);
  await waitFor(()=>expect(select.disabled).toBe(false));
  expect(screen.queryByTestId('setting-lock-note')).toBeNull();
 });
 it('is disabled with the administrator note when locked',async()=>{
  await load(true);
  render(()=><MailFeaturePreferences/>);
  const select=await screen.findByLabelText('Message list quick actions') as HTMLSelectElement;
  await waitFor(()=>expect(screen.getByTestId('setting-lock-note')).toBeInTheDocument());
  expect(select.disabled).toBe(true);
 });
});
