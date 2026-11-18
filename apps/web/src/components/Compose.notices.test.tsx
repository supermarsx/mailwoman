import {allowSendChecks} from './compose/sendChecksTestSettings.ts';
import {chooseComposeFormat} from './compose/nativeHtmlTestHarness.ts';
import {afterEach,beforeEach,expect,it,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {Compose} from './Compose.tsx';
import {makeClient} from './appHarness.tsx';
import {AppContext} from '../state/context.ts';
import {createAppState} from '../state/store.ts';
import type {Identity} from '../api/jmap-types.ts';
import type {MessageNotice} from './compose/messageNotices.ts';
const row:MessageNotice={id:'legal',title:'Legal notice',text:'LEGAL NOTICE',html:'',enabled:true,automatic:true,identities:[],kinds:['new','reply','forward'],audience:'external',domains:[],subjectContains:'Budget',bodyContains:''};
beforeEach(()=>{localStorage.clear();vi.stubGlobal('fetch',vi.fn(async()=>new Response('{}',{status:404})));});
afterEach(()=>vi.unstubAllGlobals());
async function open(patch:Partial<MessageNotice>={}){
 const identity:Identity={id:'primary',name:'Primary',email:'me@example.org',replyTo:null,sentMailboxId:null,signatureHtml:null,signatureText:'MY SIGNATURE'};
 const client=makeClient({identities:[identity]}),app=createAppState(client),close=vi.fn();await app.login({username:'me@example.org',password:'p'});await app.loadIdentities();
 await allowSendChecks({'mail.notices.templates':{value:[{...row,...patch}],source:'user',locked:false}});
 const send=vi.spyOn(app,'sendMessage').mockResolvedValue(undefined);
 render(()=><AppContext.Provider value={app}><Compose onClose={close} initial={{mode:'new',to:'you@outside.test',subject:'Budget',bodyText:'Answer',bodyHtml:'',cc:'',attachments:[],quotesDecrypted:false,inReplyTo:[],references:[]}}/></AppContext.Provider>);
 await chooseComposeFormat('plain');return {app,send};
}
it('previews applicable notices separately and sends them after the signature',async()=>{
 const state=await open();expect(await screen.findByTestId('compose-notice-preview')).toHaveTextContent('LEGAL NOTICE');
 fireEvent.click(screen.getByRole('button',{name:'Send'}));await waitFor(()=>expect(state.send).toHaveBeenCalledOnce());
 const body=state.send.mock.calls[0]![0].textBody!;expect(body.indexOf('MY SIGNATURE')).toBeGreaterThan(-1);expect(body.indexOf('LEGAL NOTICE')).toBeGreaterThan(body.indexOf('MY SIGNATURE'));
});
it('allows disabling notices for an individual composition without editing the signature',async()=>{
 const state=await open();await screen.findByTestId('compose-notice-preview');fireEvent.change(screen.getByRole('combobox',{name:'Message notices'}),{target:{value:'none'}});
 expect(screen.queryByTestId('compose-notice-preview')).toBeNull();fireEvent.click(screen.getByRole('button',{name:'Send'}));await waitFor(()=>expect(state.send).toHaveBeenCalledOnce());
 expect(state.send.mock.calls[0]![0].textBody).toContain('MY SIGNATURE');expect(state.send.mock.calls[0]![0].textBody).not.toContain('LEGAL NOTICE');
});

it('sanitizes an HTML notice and appends it after the rich signature',async()=>{
 const state=await open({html:'<p>HTML NOTICE</p><script>alert(1)</script>'});await chooseComposeFormat('html');
 fireEvent.click(screen.getByRole('button',{name:'Send'}));await waitFor(()=>expect(state.send).toHaveBeenCalledOnce());
 const html=state.send.mock.calls[0]![0].htmlBody!;expect(html).toContain('HTML NOTICE');expect(html).not.toContain('<script');expect(html.indexOf('HTML NOTICE')).toBeGreaterThan(html.indexOf('MY SIGNATURE'));
});
