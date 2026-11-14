import {beforeEach,afterEach,it,expect,vi} from 'vitest';
import {render,fireEvent,waitFor} from '@solidjs/testing-library';
import {ReaderAppearanceSettings} from './ReaderAppearance.tsx';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
import {clearSettings} from '../../api/settings.ts';
let locked=false;
let writes:Array<{key:string;value:unknown}>=[];
beforeEach(()=>{
 clearSettings();activateAccountPreferences('a','reader_settings_test');locked=false;writes=[];
 vi.stubGlobal('fetch',vi.fn(async(input:RequestInfo|URL,init?:RequestInit)=>{
  const entry=(value:unknown)=>({value,source:'global',locked});
  if(init?.method==='PUT'){const value=(JSON.parse(String(init.body)) as {value:unknown}).value;writes.push({key:String(input),value});return new Response(JSON.stringify(entry(value)));}
  return new Response(JSON.stringify({settings:{'reader.attachmentsCollapsed':entry(true),'reader.attachmentsDensity':entry('compact'),'reader.autoHideHeader':entry(false)}}));
 }));
});
afterEach(()=>{suspendAccountPreferences();vi.unstubAllGlobals();clearSettings();});
it('persists attachment defaults through policy settings independently of opening one message',async()=>{
 render(()=><ReaderAppearanceSettings/>);
 fireEvent.click(document.querySelector<HTMLButtonElement>('[role=tab][id$="-actions-tab"]')!);
 const collapsed=document.getElementById('reader-attachmentsCollapsed')!;await waitFor(()=>expect(collapsed).toBeEnabled());expect(collapsed).toBeChecked();
 fireEvent.click(collapsed);fireEvent.change(document.getElementById('reader-attachmentsDensity')!,{target:{value:'comfortable'}});
 await waitFor(()=>expect(writes).toEqual(expect.arrayContaining([{key:'/api/settings/reader.attachmentsCollapsed',value:false},{key:'/api/settings/reader.attachmentsDensity',value:'comfortable'}])));
});
it('honors administrator locks for both attachment presentation defaults',async()=>{
 locked=true;render(()=><ReaderAppearanceSettings/>);
 fireEvent.click(document.querySelector<HTMLButtonElement>('[role=tab][id$="-actions-tab"]')!);
 await waitFor(()=>expect(document.getElementById('reader-attachmentsDensity')!).toHaveValue('compact'));
 expect(document.getElementById('reader-attachmentsDensity')!).toBeDisabled();expect(document.getElementById('reader-attachmentsCollapsed')!).toBeDisabled();expect(writes).toEqual([]);
});

it('keeps automatic header hiding off by default and saves an explicit opt-in',async()=>{
 render(()=><ReaderAppearanceSettings/>);
 const toggle=document.getElementById('reader-autoHideHeader')!;await waitFor(()=>expect(toggle).toBeEnabled());expect(toggle).not.toBeChecked();
 fireEvent.click(toggle);await waitFor(()=>expect(writes).toContainEqual({key:'/api/settings/reader.autoHideHeader',value:true}));
});
it('honours the administrator lock on automatic header hiding',async()=>{
 locked=true;render(()=><ReaderAppearanceSettings/>);
 await waitFor(()=>expect(document.getElementById('reader-autoHideHeader')!).toBeDisabled());
 expect(writes).toEqual([]);
});
