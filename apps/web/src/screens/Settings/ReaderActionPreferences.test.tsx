import {afterEach,beforeEach,expect,it} from 'vitest';
import {render,screen,fireEvent,waitFor,within} from '@solidjs/testing-library';
import {ReaderActionPreferences,READER_ACTIONS_CARD} from './ReaderActionPreferences.tsx';
import {READER_ACTIONS,readerActionKey} from '../../components/readerActionPlacement.ts';
import {clearSettings,loadSettings,type SettingEntry,type Fetcher} from '../../api/settings.ts';
import {t} from '../../i18n';

let entries:Record<string,SettingEntry>,writes:Array<{key:string;method:string;value?:unknown}>,rejectWrite:boolean;
const json=(body:unknown,status=200)=>new Response(JSON.stringify(body),{status,headers:{'content-type':'application/json'}});
const fetcher:Fetcher=async(input,init)=>{
 if(!init?.method||init.method==='GET')return json({settings:entries});
 const key=decodeURIComponent(input.split('/').at(-1)!);
 const method=init.method,value=method==='PUT'?(JSON.parse(String(init.body)) as {value:unknown}).value:undefined;
 writes.push({key,method,...(method==='PUT'?{value}:{})});
 if(rejectWrite)return json({code:'invalidValue'},422);
 const action=READER_ACTIONS.find(item=>readerActionKey(item.id)===key)!;
 const entry:SettingEntry={value:method==='DELETE'?action.defaultPlacement:value,source:method==='DELETE'?'global':'user',locked:false};
 entries[key]=entry;return json(entry);
};
beforeEach(()=>{
 clearSettings();writes=[];rejectWrite=false;
 entries=Object.fromEntries(READER_ACTIONS.map(action=>[readerActionKey(action.id),{value:action.defaultPlacement,source:'global',locked:false}]));
});
afterEach(()=>clearSettings());
async function mount(){await loadSettings({refresh:true,fetcher});render(()=><ReaderActionPreferences/>);}
const control=(id:string)=>document.getElementById(`reader-action-${id}`) as HTMLSelectElement;

it('offers all canonical toolbar and menu-only actions with their existing defaults',async()=>{
 await mount();expect(screen.getAllByRole('combobox')).toHaveLength(READER_ACTIONS.length);
 expect(READER_ACTIONS_CARD.rows).toHaveLength(READER_ACTIONS.length);
 for(const action of READER_ACTIONS){
  const select=control(action.id);expect(select).toHaveAccessibleName(t(action.labelKey));expect(select).toHaveValue(action.defaultPlacement);
  expect([...select.options].map(option=>[option.value,option.textContent])).toEqual([['toolbar','Toolbar'],['menu','More menu'],['hidden','Hidden']]);
 }
 expect(control('save-contacts')).toHaveValue('menu');expect(control('reply')).toHaveValue('toolbar');
});
it('writes menu and hidden choices to the action key and resets to inherited policy',async()=>{
 await mount();const reply=control('reply'),options=[...control('reply').options];
 fireEvent.change(reply,{target:{value:'menu'}});expect(reply).toBeDisabled();
 await waitFor(()=>expect(reply).toHaveValue('menu'));await waitFor(()=>expect(reply).toBeEnabled());
 for(const [index,option] of options.entries())expect(reply.options[index]).toBe(option);
 fireEvent.change(reply,{target:{value:'hidden'}});await waitFor(()=>expect(reply).toHaveValue('hidden'));await waitFor(()=>expect(reply).toBeEnabled());
 const row=reply.closest<HTMLElement>('[data-setting-row]')!;
 fireEvent.click(within(row).getByRole('button',{name:'Use administrator default'}));
 await waitFor(()=>expect(reply).toHaveValue('toolbar'));
 expect(writes).toEqual([{key:'reader.actions.reply',method:'PUT',value:'menu'},{key:'reader.actions.reply',method:'PUT',value:'hidden'},{key:'reader.actions.reply',method:'DELETE'}]);
});
it('lets a menu-only action move onto the toolbar without changing another action',async()=>{
 await mount();const contacts=control('save-contacts');expect(contacts).toHaveValue('menu');
 fireEvent.change(contacts,{target:{value:'toolbar'}});await waitFor(()=>expect(contacts).toBeEnabled());
 expect(contacts).toHaveValue('toolbar');expect(control('reply')).toHaveValue('toolbar');
 expect(writes).toEqual([{key:'reader.actions.save-contacts',method:'PUT',value:'toolbar'}]);
});
it('holds administrator choices and reset controls without issuing a write',async()=>{
 entries['reader.actions.reply']={value:'hidden',source:'admin_user',locked:true};await mount();
 const reply=control('reply'),row=reply.closest<HTMLElement>('[data-setting-row]')!;
 expect(reply).toHaveValue('hidden');expect(reply).toBeDisabled();expect(within(row).getByRole('button',{name:'Use administrator default'})).toBeDisabled();
 expect(within(row).getByTestId('setting-lock-note')).toHaveTextContent(t('settings-policy-locked'));
 fireEvent.change(reply,{target:{value:'toolbar'}});await Promise.resolve();expect(writes).toEqual([]);
});
it('shows default placement for malformed or unknown keys and keeps unknown keys disabled',async()=>{
 entries['reader.actions.reply']={value:'elsewhere',source:'global',locked:false};delete entries['reader.actions.block-sender'];await mount();
 expect(control('reply')).toHaveValue('toolbar');expect(control('block-sender')).toHaveValue('menu');expect(control('block-sender')).toBeDisabled();expect(writes).toEqual([]);
});
it('keeps the effective placement and explains a refused save',async()=>{
 await mount();rejectWrite=true;const reply=control('reply');fireEvent.change(reply,{target:{value:'menu'}});
 const row=reply.closest<HTMLElement>('[data-setting-row]')!;
 await waitFor(()=>expect(within(row).getByRole('alert')).toHaveTextContent(t('settings-policy-error-invalid')));
 expect(reply).toHaveValue('toolbar');expect(writes).toHaveLength(1);
});
