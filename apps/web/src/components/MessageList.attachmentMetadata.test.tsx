import {afterEach,expect,it} from 'vitest';
import {cleanup,screen} from '@solidjs/testing-library';
import {MessageList} from './MessageList.tsx';
import {mkEmail,renderWithApp} from './appHarness.tsx';
import {clearSettings,loadSettings} from '../api/settings.ts';
import {groupThreads} from './threads.ts';
import type {Email} from '../api/jmap-types.ts';

const pdf={blobId:'m.1',partId:'1',type:'application/pdf',name:'iban_Habito.pdf',size:10,cid:'invoice',disposition:'inline'};
const image={...pdf,blobId:'m.2',type:'image/png',name:'logo.png'};
const mail=(id:string,attachments:unknown[])=>({...mkEmail(id,{hasAttachment:false}),attachments}) as Email;
afterEach(()=>{cleanup();clearSettings();});
for(const style of ['cards','table'])it(`shows a paperclip from real PDF metadata despite a false flag in ${style}`,async()=>{
 const {app,result}=renderWithApp(()=><MessageList/>,{emails:[mail('pdf',[pdf]),mail('image',[image]),mail('none',[])]});
 await app.login({username:'me@example.org',password:'p'});
 await loadSettings({refresh:true,fetcher:async()=>new Response(JSON.stringify({settings:{'mail.messageListStyle':{value:style,source:'user',locked:false}}}),{headers:{'content-type':'application/json'}})});
 await screen.findByText('Subject pdf');
 expect(result.container.querySelectorAll('[aria-label="Has attachment"]')).toHaveLength(1);
 expect(screen.getByText('Subject pdf').closest('.list__row')?.querySelector('[aria-label="Has attachment"]')).not.toBeNull();
});
it('keeps a conversation paperclip when only a nonrepresentative member has PDF metadata',()=>{
 const older={...mail('old',[pdf]),threadId:'t',receivedAt:'2026-01-01T00:00:00Z'};
 const newer={...mail('new',[]),threadId:'t',receivedAt:'2026-01-02T00:00:00Z'};
 const [head]=groupThreads([newer,older],new Set());
 expect(head?.kind).toBe('head');expect(head?.email.id).toBe('new');expect(head?.hasAttachment).toBe(true);
});
