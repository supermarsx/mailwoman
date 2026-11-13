import {afterEach,expect,it} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {LinkedSettingsInheritance} from './LinkedSettingsInheritance.tsx';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {loadSettings,clearSettings} from '../../api/settings.ts';
afterEach(()=>{cleanup();clearSettings();suspendAccountPreferences();});
async function mount(locked=false){activateAccountPreferences('child','child-authority');let groups:string[]=[];const writes:unknown[]=[];await loadSettings({refresh:true,fetcher:async(_url,init)=>{if(init?.method==='PUT'){groups=JSON.parse(String(init.body)).value;writes.push(groups);}return Response.json(init?.method==='PUT'?{value:groups,locked,source:'user'}:{settings:{'accounts.settings.inheritGroups':{value:groups,locked,source:'default'}}});}});render(()=><LinkedSettingsInheritance/>);return writes;}
it('defaults to independent mailbox preferences and saves explicit group inheritance',async()=>{
 const writes=await mount();const reading=await screen.findByRole('checkbox',{name:'Message reading and actions'});expect(reading).not.toBeChecked();fireEvent.click(reading);await waitFor(()=>expect(writes).toEqual([['reader']]));await waitFor(()=>expect(reading).toBeChecked());
});
it('prevents changes to administrator-held inheritance groups',async()=>{const writes=await mount(true);expect(await screen.findByRole('checkbox',{name:'Message reading and actions'})).toBeDisabled();expect(writes).toEqual([]);});
