import {afterEach,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {CachePreferences} from './CachePreferences.tsx';
const fixture=vi.hoisted(()=>({locked:false,clear:vi.fn(),save:vi.fn(async(_key:string,_value:unknown)=>true)}));
vi.mock('../../state/context.ts',()=>({useApp:()=>({clearMailCache:fixture.clear})}));
vi.mock('./useSetting.ts',()=>({useSetting:(key:string,fallback:unknown)=>({value:()=>fallback,entry:()=>({value:fallback}),locked:()=>fixture.locked,saving:()=>false,error:()=>'',reset:async()=>true,set:(value:unknown)=>fixture.save(key,value)})}));
afterEach(()=>{cleanup();fixture.locked=false;fixture.save.mockClear();fixture.clear.mockClear();});
it('keeps speculation opt-in and exposes bounded controls and exact folder exclusions',()=>{render(()=><CachePreferences/>);expect(screen.getByLabelText('Enable session cache')).toBeChecked();expect(screen.getByLabelText('Enable speculative preloading')).not.toBeChecked();expect(screen.getByLabelText('Allow speculative message bodies')).not.toBeChecked();fireEvent.change(screen.getByLabelText('Maximum simultaneous preloads'),{target:{value:'9'}});expect(fixture.save).not.toHaveBeenCalled();fireEvent.change(screen.getByLabelText('Maximum simultaneous preloads'),{target:{value:'2'}});expect(fixture.save).toHaveBeenCalledWith('mail.cache.maxConcurrency',2);fireEvent.change(screen.getByLabelText('Folders excluded from preloading'),{target:{value:'role:trash\nid:private'}});expect(fixture.save).toHaveBeenCalledWith('mail.cache.excludedFolders',['role:trash','id:private']);});
it('administrator locks disable every policy control',()=>{fixture.locked=true;render(()=><CachePreferences/>);for(const input of [...screen.getAllByRole('checkbox'),...screen.getAllByRole('spinbutton'),screen.getByRole('textbox')])expect(input).toBeDisabled();});

it('clears session memory immediately without changing stored policy',()=>{render(()=><CachePreferences/>);fireEvent.click(screen.getByRole('button',{name:'Clear cached mail now'}));expect(fixture.clear).toHaveBeenCalledOnce();expect(fixture.save).not.toHaveBeenCalled();});
