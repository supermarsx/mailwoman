import { allowSendChecks } from './compose/sendChecksTestSettings.ts';
import {chooseComposeFormat} from './compose/nativeHtmlTestHarness.ts';
import {beforeEach,afterEach,expect,it,vi} from 'vitest';
import {render,screen,fireEvent,waitFor} from '@solidjs/testing-library';
import {Compose} from './Compose.tsx';
import {makeClient} from './appHarness.tsx';
import {AppContext} from '../state/context.ts';
import {createAppState} from '../state/store.ts';
import {saveDraft} from './compose/drafts-store.ts';
import type {Identity,JmapRequest,JmapResponse} from '../api/jmap-types.ts';
import type {ComposeInitial} from './compose/reply.ts';
import {t} from '../i18n/index.ts';
const primary=(over:Partial<Identity>={}):Identity=>({id:'primary',name:'Primary',email:'me@example.org',replyTo:null,sentMailboxId:null,signatureHtml:'<p>NEW BRAND</p>',signatureText:'NEW BRAND',signatureFormat:'html',replySignatureHtml:'<table cellpadding="13"><tr><td>REPLY BRAND</td></tr></table>',replySignatureText:'REPLY BRAND',replySignatureFormat:'html',...over});
const initial=(mode:ComposeInitial['mode']='reply'):ComposeInitial=>({mode,to:'alice@example.org',cc:'',subject:'Re: Topic',bodyHtml:'<p>Answer</p>',bodyText:'Answer',inReplyTo:[],references:[],attachments:[],quotesDecrypted:false,...(mode==='reply'||mode==='reply-all'?{source:{emailId:'original',keyword:'$answered' as const}}:{})});
beforeEach(async()=>{localStorage.clear();await allowSendChecks();vi.stubGlobal('fetch',vi.fn(async()=>new Response('{}',{status:404})));});
afterEach(()=>vi.unstubAllGlobals());
async function open(identities:Identity[],seed?:ComposeInitial,resumeDraftId?:string){
 const client=makeClient({identities}),app=createAppState(client),close=vi.fn();await app.login({username:'me@example.org',password:'p'});await app.loadIdentities();
 render(()=><AppContext.Provider value={app}><Compose onClose={close} {...(seed?{initial:seed}:{})} {...(resumeDraftId?{resumeDraftId}:{})}/></AppContext.Provider>);
 await chooseComposeFormat('plain');if(!seed&&!resumeDraftId)fireEvent.input(screen.getByLabelText('To'),{target:{value:'alice@example.org'}});
 return {client,close};
}
async function send(state:Awaited<ReturnType<typeof open>>):Promise<string>{fireEvent.click(screen.getByRole('button',{name:'Send'}));await waitFor(()=>expect(state.close).toHaveBeenCalledOnce());const request=vi.mocked(state.client.jmap).mock.calls.map(call=>call[0]).find(item=>item.methodCalls.some(call=>call[0]==='EmailSubmission/set'))!;const create=request.methodCalls.find(call=>call[0]==='Email/set')![1]['create'] as Record<string,{bodyValues:{body:{value:string}}}>;return create['draft']!.bodyValues.body.value;}
it.each(['reply','reply-all'] as const)('uses the reply assignment for %s with the default sending identity',async mode=>{const state=await open([primary()],initial(mode));expect(await screen.findByTestId('compose-auto-signature')).toHaveTextContent('REPLY BRAND');const body=await send(state);expect(body).toContain('REPLY BRAND');expect(body).not.toContain('NEW BRAND');});
it.each([undefined,'forward'] as const)('uses the new-message assignment outside reply context: %s',async mode=>{const state=await open([primary()],mode?initial(mode):undefined);const body=await send(state);expect(body).toContain('NEW BRAND');expect(body).not.toContain('REPLY BRAND');});
it('keeps an explicit no-reply assignment empty',async()=>{const state=await open([primary({replySignatureHtml:null,replySignatureText:null,replySignatureFormat:'plain'})],initial());expect(screen.queryByTestId('compose-auto-signature')).toBeNull();expect(await send(state)).toBe('Answer');});
it('inherits the new assignment when reply fields are absent on an old server',async()=>{const id=primary();delete id.replySignatureHtml;delete id.replySignatureText;delete id.replySignatureFormat;const state=await open([id],initial());const body=await send(state);expect(body).toContain('NEW BRAND');expect(body).not.toContain('REPLY BRAND');});
it('uses a reply-only assignment when the new-message signature is explicitly empty',async()=>{const state=await open([primary({signatureHtml:null,signatureText:null,signatureFormat:'plain'})],initial());const body=await send(state);expect(body).toContain('REPLY BRAND');expect(body).not.toContain('NEW BRAND');});
it('uses the reply assignment after restoring saved source metadata',async()=>{saveDraft({id:'restored',accountId:'acct1',storageAuthority:'test-storage-authority-acct1',to:'alice@example.org',subject:'Saved',bodyHtml:'<p>Saved answer</p>',bodyText:'Saved answer',savedAt:1,source:{emailId:'original',keyword:'$answered'}});const state=await open([primary()],undefined,'restored');const body=await send(state);expect(body).toContain('REPLY BRAND');expect(body).not.toContain('NEW BRAND');});
it('changes the automatic reply signature with the selected sending identity',async()=>{const state=await open([primary(),primary({id:'other',name:'Other',email:'other@example.org',replySignatureHtml:null,replySignatureText:'OTHER REPLY',replySignatureFormat:'plain'})],initial());fireEvent.click(screen.getByRole('button',{name:'From'}));{const chip=screen.queryByTestId('compose-from-chip');if(chip)fireEvent.click(chip);}fireEvent.change(await screen.findByLabelText('From'),{target:{value:'other'}});expect(screen.getByTestId('compose-auto-signature')).toHaveTextContent('OTHER REPLY');const body=await send(state);expect(body).toContain('OTHER REPLY');expect(body).not.toContain('REPLY BRAND');});
it('lets a manually selected new variant replace the automatic reply variant without duplication',async()=>{const state=await open([primary()],initial());fireEvent.click(screen.getByRole('tab',{name:'Insert'}));fireEvent.click(await screen.findByTestId('signature-menu'));expect(screen.getByTestId('signature-choice-primary')).toHaveTextContent('New messages');expect(screen.getByTestId('signature-choice-primary:reply')).toHaveTextContent('Replies');fireEvent.click(screen.getByTestId('signature-choice-primary'));await waitFor(()=>expect(screen.queryByTestId('compose-auto-signature')).toBeNull());const body=await send(state);expect(body.match(/NEW BRAND/g)).toHaveLength(1);expect(body).not.toContain('REPLY BRAND');});
it('preserves native HTML layout when appending the reply variant',async()=>{const state=await open([primary()],initial());await chooseComposeFormat('html');const body=await send(state);expect(body).toContain('REPLY BRAND');expect(body).toContain('cellpadding="13"');expect(body).not.toContain('NEW BRAND');});
it('refuses a late signature upload after the sending identity changes',async()=>{
 const logo='iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==';
 vi.stubGlobal('fetch',vi.fn(async(input:RequestInfo|URL)=>String(input).endsWith('/jmap/session')?new Response(JSON.stringify(await makeClient().session()),{headers:{'content-type':'application/json'}}):new Response('{}',{status:404})));
 Object.assign(URL,{createObjectURL:vi.fn(()=>'blob:signature-preview'),revokeObjectURL:vi.fn()});
 const state=await open([primary({replySignatureHtml:`<p>REPLY BRAND<img src="data:image/png;base64,${logo}"></p>`}),primary({id:'other',name:'Other',email:'other@example.org',replySignatureHtml:null,replySignatureText:'OTHER REPLY',replySignatureFormat:'plain'})],initial());
 await chooseComposeFormat('html');let complete!:(response:Response)=>void;const original=globalThis.fetch;
 vi.stubGlobal('fetch',vi.fn(async(input:RequestInfo|URL,init?:RequestInit)=>String(input).endsWith('/u')?new Promise<Response>(resolve=>{complete=resolve;}):original(input,init)));
 fireEvent.click(screen.getByRole('button',{name:'Send'}));await waitFor(()=>expect(complete).toBeDefined());
 fireEvent.click(screen.getByRole('button',{name:'From'}));{const chip=screen.queryByTestId('compose-from-chip');if(chip)fireEvent.click(chip);}fireEvent.change(await screen.findByLabelText('From'),{target:{value:'other'}});
 complete(new Response(JSON.stringify({accountId:'acct1',blobId:'signature-upload',type:'image/png',size:68}),{headers:{'content-type':'application/json'}}));
 await waitFor(()=>expect(screen.getByRole('button',{name:'Send'})).toBeEnabled());expect(screen.getByRole('alert')).toHaveTextContent(t('mail-compose-send-failed'));expect(state.close).not.toHaveBeenCalled();expect(vi.mocked(state.client.jmap).mock.calls.some(call=>call[0].methodCalls.some(method=>method[0]==='EmailSubmission/set'))).toBe(false);
});
it.each(['ready','failed','changed'] as const)('waits for the original identity request before a fast reply Send: %s',async outcome=>{
 const client=makeClient({identities:[primary({email:' ME@EXAMPLE.ORG '})]}),app=createAppState(client),close=vi.fn();await app.login({username:'me@example.org',password:'p'});
 const original=client.jmap;let release!:(value:JmapResponse)=>void,reject!:(cause:Error)=>void,captured!:JmapRequest,calls=0;
 client.jmap=vi.fn(async(request:JmapRequest,options?:{signal?:AbortSignal})=>{
  if(request.methodCalls.some(call=>call[0]==='Identity/get')){captured=request;calls++;return new Promise<JmapResponse>((resolve,fail)=>{release=resolve;reject=fail;});}
  return original(request,options);
 });
 render(()=><AppContext.Provider value={app}><Compose onClose={close} initial={initial()}/></AppContext.Provider>);await chooseComposeFormat('plain');
 await waitFor(()=>expect(release).toBeDefined());const submit=screen.getByRole('button',{name:'Send'});fireEvent.click(submit);await waitFor(()=>expect(submit).toBeDisabled());
 expect(calls).toBe(1);expect(vi.mocked(client.jmap).mock.calls.some(call=>call[0].methodCalls.some(method=>method[0]==='EmailSubmission/set'))).toBe(false);
 if(outcome==='changed'){fireEvent.click(screen.getByRole('button',{name:'From'}));{const chip=screen.queryByTestId('compose-from-chip');if(chip)fireEvent.click(chip);}fireEvent.change(await screen.findByLabelText('From'),{target:{value:'__custom_from__'}});}
 if(outcome==='failed')reject(new Error('Offline'));else release(await original(captured));
 if(outcome==='ready'){
  await waitFor(()=>expect(close).toHaveBeenCalledOnce());const request=vi.mocked(client.jmap).mock.calls.map(call=>call[0]).find(item=>item.methodCalls.some(call=>call[0]==='EmailSubmission/set'))!;const draft=(request.methodCalls.find(call=>call[0]==='Email/set')![1]['create'] as Record<string,{bodyValues:{body:{value:string}}}>)['draft']!;expect(draft.bodyValues.body.value).toContain('REPLY BRAND');expect(draft.bodyValues.body.value).not.toContain('NEW BRAND');
 }else{
  await waitFor(()=>expect(screen.getByRole('button',{name:'Send'})).toBeEnabled());expect(close).not.toHaveBeenCalled();expect(screen.getByRole('alert')).toHaveTextContent(t(outcome==='failed'?'mail-compose-identities-failed':'mail-compose-send-failed'));expect(vi.mocked(client.jmap).mock.calls.some(call=>call[0].methodCalls.some(method=>method[0]==='EmailSubmission/set'))).toBe(false);
 }
});
