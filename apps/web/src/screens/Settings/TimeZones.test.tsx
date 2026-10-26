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
