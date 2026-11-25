import {LANGUAGE_CARD} from './Language.tsx';
import { beforeEach, expect, it, vi } from 'vitest';
import { fireEvent, render, screen, waitFor } from '@solidjs/testing-library';
import { TimeZones } from './TimeZones.tsx';
import { createSettingsNavigation } from './navigation.tsx';
const data=vi.hoisted(()=>({values:{} as Record<string,{value:unknown;locked:boolean}>,save:vi.fn(),reset:vi.fn()}));
vi.mock('./useSetting.ts',()=>({useSetting:(key:string,fallback:unknown)=>({value:()=>data.values[key]?.value??fallback,entry:()=>data.values[key],locked:()=>data.values[key]?.locked===true,saving:()=>false,error:()=>'',set:(value:unknown)=>data.save(key,value),reset:()=>data.reset(key)})}));
beforeEach(()=>{data.values={'time.displayZone':{value:'local',locked:false},'time.comparisonZone':{value:'',locked:false},'time.showHelpers':{value:true,locked:false}};for(const key of ['region','dateFormat','timeFormat','hourCycle','firstDayOfWeek'])data.values['time.'+key]={value:'auto',locked:false};data.save.mockReset().mockResolvedValue(true);data.reset.mockReset();});
it('searches time zones and immediately saves only a chosen option',async()=>{
 render(()=><TimeZones navigation={createSettingsNavigation({section:'time'})}/>);
 const input=screen.getByLabelText('Comparison time zone');fireEvent.focus(input);
 fireEvent.input(input,{target:{value:'person@example.test'}});expect(data.save).not.toHaveBeenCalled();expect(screen.queryAllByRole('option')).toHaveLength(0);
 fireEvent.input(input,{target:{value:'New York'}});fireEvent.click(await screen.findByRole('option',{name:/New York/}));
 await waitFor(()=>expect(data.save).toHaveBeenCalledWith('time.comparisonZone','America/New_York'));
});
it('honors administrative locks and offers inherited defaults',()=>{
 data.values['time.displayZone']!.locked=true;
 render(()=><TimeZones navigation={createSettingsNavigation({section:'time'})}/>);
 expect(screen.getByLabelText('Display time zone')).toBeDisabled();
 const comparison=screen.getByLabelText('Comparison time zone').closest('[data-setting-row]')!;
 fireEvent.click(comparison.querySelector('button[type="button"]:not([aria-controls])')!);
 expect(data.reset).toHaveBeenCalledWith('time.comparisonZone');
});
it('keeps searches local until a regional option is chosen',async()=>{
 const navigation=createSettingsNavigation({section:'time'});render(()=><TimeZones navigation={navigation}/>);
 const region=await screen.findByLabelText('Region');fireEvent.focus(region);fireEvent.input(region,{target:{value:'Portugal'}});
 expect(data.save).not.toHaveBeenCalled();expect(navigation.isDirty()).toBe(false);
 fireEvent.click(await screen.findByRole('option',{name:'Portugal'}));expect(data.save).toHaveBeenCalledWith('time.region','PT');
 const clock=screen.getByLabelText('Clock');fireEvent.focus(clock);fireEvent.keyDown(clock,{key:'End'});fireEvent.keyDown(clock,{key:'Enter'});
 expect(data.save).toHaveBeenCalledWith('time.hourCycle','24');
});
it('honours regional administrator locks',async()=>{
 data.values['time.dateFormat']!.locked=true;render(()=><TimeZones navigation={createSettingsNavigation({section:'time'})}/>);
 expect(await screen.findByLabelText('Date format')).toBeDisabled();expect(data.save).not.toHaveBeenCalled();
});
it('saves message date choices and honours their administrator locks',async()=>{
 data.values['time.messageYear']={value:'auto',locked:false};data.values['time.messageShowTime']={value:true,locked:false};
 const view=render(()=><TimeZones navigation={createSettingsNavigation({section:'time'})}/>);
 const year=await screen.findByLabelText('Year in message dates');fireEvent.focus(year);fireEvent.keyDown(year,{key:'End'});fireEvent.keyDown(year,{key:'Enter'});
 expect(data.save).toHaveBeenCalledWith('time.messageYear','never');
 fireEvent.click(screen.getByLabelText('Show time in message dates'));expect(data.save).toHaveBeenCalledWith('time.messageShowTime',false);
 view.unmount();data.values['time.messageYear']!.locked=true;data.values['time.messageShowTime']!.locked=true;
 render(()=><TimeZones navigation={createSettingsNavigation({section:'time'})}/>);
 expect(screen.getByLabelText('Year in message dates')).toBeDisabled();expect(screen.getByLabelText('Show time in message dates')).toBeDisabled();
});

it('keeps language in Regional settings and indexes it under the same section',()=>{
 render(()=><TimeZones navigation={createSettingsNavigation({section:'time'})}/>);
 expect(screen.getByLabelText('Language')).toBeVisible();expect(LANGUAGE_CARD.section).toBe('time');
});
