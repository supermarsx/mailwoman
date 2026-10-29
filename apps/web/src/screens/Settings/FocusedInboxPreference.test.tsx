import {afterEach,beforeEach,expect,it} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {FocusedInboxPreference} from './FocusedInboxPreference.tsx';
import {clearSettings,loadSettings,settings,type Fetcher} from '../../api/settings.ts';
// t73: the Focused inbox switch lives in Settings > Folders and sidebar.
const json=(body:unknown,status=200)=>new Response(JSON.stringify(body),{status,headers:{'content-type':'application/json'}});
let puts:unknown[]=[];
function serve(entry:Record<string,unknown>):Promise<void>{
 return loadSettings({refresh:true,fetcher:(async(_url:string,init?:RequestInit)=>{
  if(init?.method==='PUT'){const body=JSON.parse(String(init.body)) as {value:unknown};puts.push(body);return json({value:body.value,source:'user',locked:false});}
  return json({settings:{'mail.focusedInbox':entry}});
 }) as Fetcher});
}
beforeEach(()=>{clearSettings();puts=[];});
afterEach(()=>{cleanup();clearSettings();});
it('is off by default and turning it on stores the preference in the shared map',async()=>{
 await serve({value:false,source:'default',locked:false});
 render(()=><FocusedInboxPreference/>);
 const box=screen.getByLabelText('Focused inbox');
 expect(box).not.toBeChecked();expect(box).toBeEnabled();
 fireEvent.click(box);
 await waitFor(()=>expect(settings()['mail.focusedInbox']?.value).toBe(true));
 expect(puts).toEqual([{value:true}]);
 expect(screen.getByLabelText('Focused inbox')).toBeChecked();
});
it('follows an administrator default and is disabled while locked',async()=>{
 await serve({value:true,source:'global',locked:true,lockedBy:'global'});
 render(()=><FocusedInboxPreference/>);
 const box=screen.getByLabelText('Focused inbox');
 expect(box).toBeChecked();expect(box).toBeDisabled();
 expect(screen.getByTestId('focused-inbox-preference').textContent).toMatch(/administrator/i);
 fireEvent.click(box);expect(puts).toEqual([]);
});
