import { beforeEach, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { TimeZones } from './TimeZones.tsx';
import { createSettingsNavigation } from './navigation.tsx';
const data=vi.hoisted(()=>({values:{} as Record<string,{value:unknown;locked:boolean}>,save:vi.fn(),reset:vi.fn()}));
vi.mock('./useSetting.ts',()=>({useSetting:(key:string,fallback:unknown)=>({value:()=>data.values[key]?.value??fallback,entry:()=>data.values[key],locked:()=>data.values[key]?.locked===true,saving:()=>false,error:()=>'',set:(value:unknown)=>data.save(key,value),reset:()=>data.reset(key)})}));
beforeEach(()=>{data.values={'time.displayZone':{value:'local',locked:false},'time.comparisonZone':{value:'',locked:false},'time.showHelpers':{value:true,locked:false}};data.save.mockReset().mockResolvedValue(true);data.reset.mockReset();});
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
 fireEvent.click(comparison.querySelector('button[type="button"]')!);
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
