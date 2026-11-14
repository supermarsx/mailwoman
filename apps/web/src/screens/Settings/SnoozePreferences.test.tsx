import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {SnoozePreferences} from './SnoozePreferences.tsx';
import {DEFAULT_SNOOZE_PRESETS,SNOOZE_PRESETS_KEY,FOLLOW_UP_PRESETS_KEY} from '../../state/snooze.ts';
import {clearSettings} from '../../api/settings.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
let locked=false;
const writes:unknown[]=[];
const paths:string[]=[];
beforeEach(()=>{
 clearSettings();activateAccountPreferences('a','authority_snooze_settings');locked=false;writes.length=0;paths.length=0;
 vi.stubGlobal('fetch',vi.fn(async(_input:unknown,init?:RequestInit)=>{
  const entry=(value:unknown)=>({value,source:'global',locked});
  if(init?.method==='PUT'){const {value}=JSON.parse(String(init.body)) as {value:unknown};writes.push(value);paths.push(String(_input));return new Response(JSON.stringify(entry(value)));}
  return new Response(JSON.stringify({settings:{[SNOOZE_PRESETS_KEY]:entry(DEFAULT_SNOOZE_PRESETS),[FOLLOW_UP_PRESETS_KEY]:entry(DEFAULT_SNOOZE_PRESETS)}}));
 }));
});
afterEach(()=>{suspendAccountPreferences();clearSettings();vi.unstubAllGlobals();});
it('saves a custom menu only after the user saves the edited presets',async()=>{
 render(()=><SnoozePreferences/>);
 const add=screen.getByRole('button',{name:'Add preset'});await waitFor(()=>expect(add).toBeEnabled());fireEvent.click(add);
 fireEvent.input(screen.getByLabelText('Name (optional)'),{target:{value:'After lunch'}});
 fireEvent.input(screen.getByLabelText('Amount'),{target:{value:'6'}});
 expect(writes).toEqual([]);fireEvent.click(screen.getByRole('button',{name:'Save snooze options'}));
 await waitFor(()=>expect(writes).toHaveLength(1));
 expect((writes[0] as unknown[]).at(-1)).toMatchObject({kind:'hours',label:'After lunch',amount:6});
});
it('disables preset editing when an administrator locks the effective setting',async()=>{
 locked=true;render(()=><SnoozePreferences/>);
 await waitFor(()=>expect(screen.getByTestId('setting-lock-note')).toBeInTheDocument());
 expect(screen.getByRole('button',{name:'Add preset'})).toBeDisabled();expect(screen.getByRole('button',{name:'Save snooze options'})).toBeDisabled();
 expect(screen.getByRole('checkbox',{name:'Show in menu: Tomorrow'})).toBeDisabled();
});

it('saves follow-up presets independently and applies administrator locks to that editor',async()=>{
 const view=render(()=><SnoozePreferences followUp/>);
 const add=screen.getByRole('button',{name:'Add preset'});await waitFor(()=>expect(add).toBeEnabled());fireEvent.click(add);
 fireEvent.click(screen.getByRole('button',{name:'Save'}));
 await waitFor(()=>expect(paths).toHaveLength(1));expect(paths[0]).toContain(FOLLOW_UP_PRESETS_KEY);
 view.unmount();locked=true;clearSettings();render(()=><SnoozePreferences followUp/>);
 await waitFor(()=>expect(screen.getByTestId('setting-lock-note')).toBeInTheDocument());
 expect(screen.getByRole('button',{name:'Add preset'})).toBeDisabled();
});
