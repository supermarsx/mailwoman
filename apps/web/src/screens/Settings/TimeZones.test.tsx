import {LANGUAGE_CARD} from './Language.tsx';
import { beforeEach, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { TimeZones } from './TimeZones.tsx';
import { createSettingsNavigation } from './navigation.tsx';
const data=vi.hoisted(()=>({values:{} as Record<string,{value:unknown;locked:boolean}>,save:vi.fn(),reset:vi.fn()}));
vi.mock('./useSetting.ts',()=>({useSetting:(key:string,fallback:unknown)=>({value:()=>data.values[key]?.value??fallback,entry:()=>data.values[key],locked:()=>data.values[key]?.locked===true,saving:()=>false,error:()=>'',set:(value:unknown)=>data.save(key,value),reset:()=>data.reset(key)})}));
beforeEach(()=>{data.values={'time.displayZone':{value:'local',locked:false},'time.comparisonZone':{value:'',locked:false},'time.showHelpers':{value:true,locked:false}};for(const key of ['region','dateFormat','timeFormat','hourCycle','firstDayOfWeek'])data.values['time.'+key]={value:'auto',locked:false};data.save.mockReset().mockResolvedValue(true);data.reset.mockReset();});
it('validates and saves an explicit comparison through account settings',async()=>{
 render(()=><TimeZones navigation={createSettingsNavigation({section:'time'})}/>);
 const input=screen.getByLabelText('Comparison time zone');
 fireEvent.input(input,{target:{value:'person@example.test'}});fireEvent.submit(input.closest('form')!);
 expect(screen.getByRole('alert')).toHaveTextContent('valid IANA');expect(data.save).not.toHaveBeenCalled();
 fireEvent.input(input,{target:{value:'America/New_York'}});fireEvent.submit(input.closest('form')!);
 await waitFor(()=>expect(data.save).toHaveBeenCalledWith('time.comparisonZone','America/New_York'));
});
it('honors administrative locks and offers inherited defaults',()=>{
 data.values['time.displayZone']!.locked=true;
 render(()=><TimeZones navigation={createSettingsNavigation({section:'time'})}/>);
 expect(screen.getByLabelText('Display time zone')).toBeDisabled();
 const comparison=screen.getByLabelText('Comparison time zone').closest('form')!;
 fireEvent.click(comparison.querySelector('button[type="button"]:not([aria-controls])')!);
 expect(data.reset).toHaveBeenCalledWith('time.comparisonZone');
});
it('asks to discard in the in-app dialog, never window.confirm, and a bulk close that already chose discard does not ask again (t67)',async()=>{
 const native=vi.spyOn(window,'confirm').mockReturnValue(true);
 const navigation=createSettingsNavigation({section:'time'});
 render(()=><TimeZones navigation={navigation}/>);
 const input=screen.getByLabelText('Comparison time zone');
 fireEvent.input(input,{target:{value:'America/New_York'}});
 expect(navigation.isDirty()).toBe(true);
 // A bulk tab close lists this tab in its own dialog, then closes with discard: no second dialog.
 await expect(navigation.canLeave(true)).resolves.toBe(true);
 expect(screen.queryByRole('alertdialog')).toBeNull();
 // A plain navigation asks once, in the shared ConfirmDialog.
 const leaving=navigation.canLeave();
 const dialog=await screen.findByRole('alertdialog');
 expect(dialog).toHaveTextContent('Discard unsaved time zone changes?');
 fireEvent.click(screen.getByRole('button',{name:'Keep editing'}));
 await expect(leaving).resolves.toBe(false);
 const again=navigation.canLeave();
 fireEvent.click(await screen.findByRole('button',{name:'Discard changes'}));
 await expect(again).resolves.toBe(true);
 expect(native).not.toHaveBeenCalled();
 native.mockRestore();
});

it('keeps regional defaults automatic, validates patterns and saves explicit preferences',async()=>{
 render(()=><TimeZones navigation={createSettingsNavigation({section:'time'})}/>);
 const date=await screen.findByLabelText('Date format');expect(date).toHaveValue('auto');
 fireEvent.input(date,{target:{value:'YYYY-MM-DD'}});fireEvent.submit(date.closest('form')!);expect(await screen.findByRole('alert')).toHaveTextContent('supported pattern');expect(data.save).not.toHaveBeenCalled();
 fireEvent.input(date,{target:{value:'dd/MM/yyyy'}});fireEvent.submit(date.closest('form')!);await waitFor(()=>expect(data.save).toHaveBeenCalledWith('time.dateFormat','dd/MM/yyyy'));
 const region=screen.getByLabelText('Region');fireEvent.input(region,{target:{value:'pt'}});fireEvent.submit(region.closest('form')!);await waitFor(()=>expect(data.save).toHaveBeenCalledWith('time.region','PT'));
 fireEvent.change(screen.getByLabelText('Clock'),{target:{value:'24'}});expect(data.save).toHaveBeenCalledWith('time.hourCycle','24');
});
it('honours regional administrator locks',async()=>{
 data.values['time.dateFormat']!.locked=true;render(()=><TimeZones navigation={createSettingsNavigation({section:'time'})}/>);
 expect(await screen.findByLabelText('Date format')).toBeDisabled();expect(data.save).not.toHaveBeenCalled();
});

it('keeps language in Regional settings and indexes it under the same section',()=>{
 render(()=><TimeZones navigation={createSettingsNavigation({section:'time'})}/>);
 expect(screen.getByLabelText('Language')).toBeVisible();expect(LANGUAGE_CARD.section).toBe('time');
});
