import {afterEach,expect,it,vi} from 'vitest';
import {fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {activateAccountPreferences,suspendAccountPreferences} from '../api/accountPreferenceLifecycle.ts';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {requestCustomSnooze,SnoozeDialog} from './SnoozeDialog.tsx';
afterEach(()=>{suspendAccountPreferences();clearSettings();vi.useRealTimers();});
async function mount(){
 vi.useFakeTimers({toFake:['Date']});vi.setSystemTime(new Date('2026-10-24T12:00:00Z'));activateAccountPreferences('a','authority_snooze_123');
 await loadSettings({fetcher:async()=>new Response(JSON.stringify({settings:{'time.displayZone':{value:'Europe/London',source:'user',locked:false}}}))});
 const save=vi.fn(async()=>{});render(()=><SnoozeDialog/>);requestCustomSnooze(save);return save;
}
it('rejects past times and requires an explicit occurrence in the repeated hour',async()=>{
 const save=await mount();const input=screen.getByLabelText('Choose date and time…',{selector:'input'});
 fireEvent.input(input,{target:{value:'2026-10-23T10:00'}});fireEvent.click(screen.getByRole('button',{name:'Snooze'}));expect(screen.getByRole('alert')).toBeVisible();expect(save).not.toHaveBeenCalled();
 fireEvent.input(input,{target:{value:'2026-10-25T01:30'}});fireEvent.click(screen.getByRole('button',{name:'Snooze'}));expect(save).not.toHaveBeenCalled();
 fireEvent.change(screen.getByRole('combobox'),{target:{value:'2026-10-25T01:30:00.000Z'}});fireEvent.click(screen.getByRole('button',{name:'Snooze'}));
 await waitFor(()=>expect(save).toHaveBeenCalledWith('2026-10-25T01:30:00.000Z'));
});
it('closes without saving when the mailbox changes',async()=>{
 const save=await mount();suspendAccountPreferences();expect(screen.queryByRole('dialog')).not.toBeInTheDocument();expect(save).not.toHaveBeenCalled();
});
