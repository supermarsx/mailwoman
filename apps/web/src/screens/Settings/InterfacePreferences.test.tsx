import {afterEach, expect, it, vi} from 'vitest';
import {fireEvent, render, screen, waitFor} from '@solidjs/testing-library';
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
