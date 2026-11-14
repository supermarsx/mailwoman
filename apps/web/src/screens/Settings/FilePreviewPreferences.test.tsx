import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {FilePreviewPreferences} from './FilePreviewPreferences.tsx';
import {DEFAULT_FILE_PREVIEWS,FILE_PREVIEWS_KEY} from '../../viewers/filePreviewPreferences.ts';
import {clearSettings} from '../../api/settings.ts';
import {activateAccountPreferences,suspendAccountPreferences} from '../../api/accountPreferenceLifecycle.ts';
let locked=false;const writes:Array<{path:string;value:unknown;method:string}>=[];
beforeEach(()=>{
 clearSettings();activateAccountPreferences('a','preview_settings_test');locked=false;writes.length=0;
 vi.stubGlobal('fetch',vi.fn(async(input:unknown,init?:RequestInit)=>{
  const entry=(value:unknown)=>({value,source:'global',locked});
  if(init?.method==='PUT'||init?.method==='DELETE'){const value=init.method==='PUT'?(JSON.parse(String(init.body)) as {value:unknown}).value:DEFAULT_FILE_PREVIEWS;writes.push({path:String(input),value,method:init.method});return new Response(JSON.stringify(entry(value)));}
  return new Response(JSON.stringify({settings:{[FILE_PREVIEWS_KEY]:entry(DEFAULT_FILE_PREVIEWS)}}));
 }));
});
afterEach(()=>{suspendAccountPreferences();clearSettings();vi.unstubAllGlobals();});
it('saves format choices only when requested and can restore the inherited default',async()=>{
 render(()=><FilePreviewPreferences/>);
 const pdf=await screen.findByRole('checkbox',{name:'PDF documents'});await waitFor(()=>expect(pdf).toBeEnabled());fireEvent.click(pdf);
 expect(writes).toEqual([]);fireEvent.click(screen.getByRole('button',{name:'Save preview settings'}));
 await waitFor(()=>expect(writes).toHaveLength(1));expect(writes[0]).toMatchObject({path:'/api/settings/'+FILE_PREVIEWS_KEY,value:{types:{pdf:false}},method:'PUT'});
 await waitFor(()=>expect(screen.getByRole('button',{name:'Use administrator default'})).toBeEnabled());fireEvent.click(screen.getByRole('button',{name:'Use administrator default'}));await waitFor(()=>expect(writes.at(-1)?.method).toBe('DELETE'));
});
it('shows administrator defaults and disables every edit when locked',async()=>{
 locked=true;render(()=><FilePreviewPreferences/>);
 await waitFor(()=>expect(screen.getByTestId('setting-lock-note')).toBeInTheDocument());
 expect(screen.getByRole('checkbox',{name:'PDF documents'})).toBeDisabled();expect(screen.getByRole('button',{name:'Save preview settings'})).toBeDisabled();expect(writes).toEqual([]);
});
