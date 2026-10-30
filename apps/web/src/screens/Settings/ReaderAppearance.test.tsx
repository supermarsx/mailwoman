import {beforeEach,afterEach,it,expect,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {ReaderAppearanceSettings} from './ReaderAppearance.tsx';
import {clearSettings} from '../../api/settings.ts';
let locked=false;
let writes:Array<{key:string;value:unknown}>=[];
beforeEach(()=>{
 clearSettings();locked=false;writes=[];
 vi.stubGlobal('fetch',vi.fn(async(input:RequestInfo|URL,init?:RequestInit)=>{
  const entry=(value:unknown)=>({value,source:'global',locked});
  if(init?.method==='PUT'){const value=(JSON.parse(String(init.body)) as {value:unknown}).value;writes.push({key:String(input),value});return new Response(JSON.stringify(entry(value)));}
  return new Response(JSON.stringify({settings:{'reader.attachmentsCollapsed':entry(true),'reader.attachmentsDensity':entry('compact')}}));
 }));
});
afterEach(()=>{vi.unstubAllGlobals();clearSettings();});
it('persists attachment defaults through policy settings independently of opening one message',async()=>{
 render(()=><ReaderAppearanceSettings/>);
 const collapsed=await screen.findByLabelText('Collapse attachments by default');await waitFor(()=>expect(collapsed).toBeEnabled());expect(collapsed).toBeChecked();
 fireEvent.click(collapsed);fireEvent.change(screen.getByLabelText('Attachment density'),{target:{value:'comfortable'}});
 await waitFor(()=>expect(writes).toEqual(expect.arrayContaining([{key:'/api/settings/reader.attachmentsCollapsed',value:false},{key:'/api/settings/reader.attachmentsDensity',value:'comfortable'}])));
});
it('honors administrator locks for both attachment presentation defaults',async()=>{
 locked=true;render(()=><ReaderAppearanceSettings/>);
 await waitFor(()=>expect(screen.getByLabelText('Attachment density')).toHaveValue('compact'));
 expect(screen.getByLabelText('Attachment density')).toBeDisabled();expect(screen.getByLabelText('Collapse attachments by default')).toBeDisabled();expect(writes).toEqual([]);
});
