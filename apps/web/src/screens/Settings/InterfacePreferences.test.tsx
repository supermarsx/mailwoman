import {afterEach, expect, it, vi} from 'vitest';
import {fireEvent, render, screen, waitFor} from '@solidjs/testing-library';
import {loadCatalog, t} from '../../i18n/index.ts';
import {MESSAGE_LIST_GROUPS, MESSAGE_LIST_ORDERS} from '../../components/messageOrder.ts';
import {settingLabel, settingHelp} from '../Admin/Settings/labels.ts';
import {INTERFACE_CARD} from './InterfacePreferences.tsx';
import {clearSettings, loadSettings} from '../../api/settings.ts';
import {InterfacePreferences} from './InterfacePreferences.tsx';
afterEach(()=>clearSettings());
it('persists message presentation options while enforcing administrator text locks',async()=>{
 const fetcher=vi.fn(async(_url:unknown,init?:RequestInit)=>new Response(JSON.stringify(init?.method==='PUT'?{value:JSON.parse(String(init.body)).value,source:'user',locked:false}:{settings:{'appearance.fontSize':{value:18,source:'global',locked:true},'mail.messageListStyle':{value:'cards',source:'default',locked:false},'reader.attachmentsAtEnd':{value:false,source:'default',locked:false}}})));
 await loadSettings({fetcher});render(()=><InterfacePreferences/>);
 expect(await screen.findByLabelText('Interface font size')).toBeDisabled();const style=screen.getByLabelText('Message list style');await waitFor(()=>expect(style).toBeEnabled());
 fireEvent.change(style,{target:{value:'table'}});await waitFor(()=>expect(fetcher).toHaveBeenCalledWith('/api/settings/mail.messageListStyle',expect.objectContaining({method:'PUT',body:JSON.stringify({value:'table'})})));
 expect(screen.getByLabelText('Show attachments at the end of messages')).not.toBeChecked();
});

it('renders complete grouping and sorting labels, help, and choices without raw translation ids',async()=>{
 await loadCatalog('appearance');await loadCatalog('admin-settings');
 const fetcher=vi.fn(async(_url:unknown,init?:RequestInit)=>new Response(JSON.stringify(init?.method==='PUT'?{value:JSON.parse(String(init.body)).value,source:'user',locked:false}:{settings:{'mail.messageListGroup':{value:'none',source:'default',locked:false},'mail.messageListOrder':{value:'newest',source:'default',locked:false}}})));
 await loadSettings({fetcher});const {container}=render(()=><InterfacePreferences/>);
 const group=await screen.findByRole('combobox',{name:'Message grouping'});
 expect(group).toHaveValue('none');
 expect([...group.querySelectorAll('option')].map(option=>option.textContent)).toEqual(['No grouping','Received date','Sent date','Sender','Subject','Importance','Message size']);
 expect([...group.querySelectorAll('option')].map(option=>option.value)).toEqual([...MESSAGE_LIST_GROUPS]);
 expect(group).toHaveAccessibleDescription(/Keep related messages together under group headings/);
 const order=screen.getByRole('combobox',{name:'Message list order'});
 expect([...order.querySelectorAll('option')].map(option=>option.value)).toEqual([...MESSAGE_LIST_ORDERS]);
 expect(order).toHaveAccessibleDescription(/order of messages within each group/);
 expect(container.textContent).not.toContain('appearance-preference-');
 for(const row of INTERFACE_CARD.rows) {
   expect(t(row.label)).not.toBe(row.label);
   if(row.help) expect(t(row.help)).not.toBe(row.help);
 }
 expect(settingLabel('mail.messageListGroup')).toBe('Message grouping');
 expect(settingHelp('mail.messageListGroup')).toContain('importance or size');
 expect(settingHelp('mail.messageListOrder')).toContain('importance, or size');
 fireEvent.change(group,{target:{value:'importance'}});
 await waitFor(()=>expect(fetcher).toHaveBeenCalledWith('/api/settings/mail.messageListGroup',expect.objectContaining({method:'PUT',body:JSON.stringify({value:'importance'})})));
});
it('keeps grouping disabled under an administrator lock',async()=>{
 await loadCatalog('appearance');
 await loadSettings({fetcher:async()=>new Response(JSON.stringify({settings:{'mail.messageListGroup':{value:'sender',source:'global',locked:true}}}))});
 render(()=><InterfacePreferences/>);
 const group=await screen.findByRole('combobox',{name:'Message grouping'});
 expect(group).toHaveValue('sender');expect(group).toBeDisabled();
});
