import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import {chooseComposeFormat} from './compose/nativeHtmlTestHarness.ts';
import {Compose} from './Compose.tsx';
import {makeClient} from './appHarness.tsx';
import {createAppState} from '../state/store.ts';
import {AppContext} from '../state/context.ts';
import {clearSettings,loadSettings,type SettingSource} from '../api/settings.ts';
import {cardForAddress} from '../modules/contacts/share.ts';
import {recipientList} from './compose/recipientTestUtils.ts';

beforeEach(()=>{clearSettings();localStorage.clear();});
afterEach(()=>clearSettings());

async function policy(value:boolean,source:SettingSource='user',locked=false){
 await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{'mail.compose.autoAddMentionRecipients':{value,source,locked}}}),{headers:{'content-type':'application/json'}})});
}
async function mount(initial?:{to?:string;cc?:string;bcc?:string}){
 const app=createAppState(makeClient());await app.login({username:'me@example.org',password:'p'});
 vi.spyOn(app,'contacts').mockReturnValue([cardForAddress('jane@example.org','Jane Doe')]);
 render(()=><AppContext.Provider value={app}><Compose onClose={()=>undefined}/></AppContext.Provider>);
 await chooseComposeFormat('plain');
 if(initial){
  fireEvent.click(screen.getByTestId('compose-show-cc-bcc'));
  for(const field of ['To','Cc','Bcc'] as const){
   const value=initial[field.toLowerCase() as 'to'|'cc'|'bcc'];
   if(value)fireEvent.input(screen.getByRole('combobox',{name:field}),{target:{value}});
  }
 }
 return screen.getByLabelText('Body') as HTMLTextAreaElement;
}
async function mention(area:HTMLTextAreaElement){
 area.focus();fireEvent.input(area,{target:{value:'Hello @Ja'}});area.setSelectionRange(9,9);fireEvent.input(area);
 const option=await screen.findByRole('option',{name:/Jane Doe/});fireEvent.click(option);
 await waitFor(()=>expect(area.value).toBe('Hello @jane@example.org '));
}
it('inserts a mention without adding a recipient by default',async()=>{
 const area=await mount();await mention(area);
 expect(recipientList(screen.getByRole('combobox',{name:'To'}))).toBe('');
});
it.each(['user','global'] as const)('adds a selected mention with an effective %s opt-in',async source=>{
 const area=await mount();await policy(true,source,source==='global');await mention(area);
 expect(recipientList(screen.getByRole('combobox',{name:'To'}))).toContain('jane@example.org');
});
it('honours a locked administrator opt-out',async()=>{
 const area=await mount();await policy(false,'global',true);await mention(area);
 expect(recipientList(screen.getByRole('combobox',{name:'To'}))).toBe('');
});
it('does not duplicate a mention already in Cc',async()=>{
 const area=await mount({cc:'Jane <JANE@example.org>'});await policy(true);await mention(area);
 expect(recipientList(screen.getByRole('combobox',{name:'To'}))).toBe('');
 expect(recipientList(screen.getByRole('combobox',{name:'Cc'}))).toContain('JANE@example.org');
});
it('typing a mention without selecting a suggestion never adds recipients',async()=>{
 const area=await mount();await policy(true);fireEvent.input(area,{target:{value:'Hi @jane@example.org'}});
 expect(recipientList(screen.getByRole('combobox',{name:'To'}))).toBe('');
});
