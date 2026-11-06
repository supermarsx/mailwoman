import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor,within} from '@solidjs/testing-library';
import {makeClient,mkEmail} from '../appHarness.tsx';
import {createAppState} from '../../state/store.ts';
import {AppContext} from '../../state/context.ts';
import {clearSettings,loadSettings,type Fetcher} from '../../api/settings.ts';
import type {Email,JmapRequest,JmapResponse} from '../../api/jmap-types.ts';
import {MessageDialogs} from './MessageProperties.tsx';
import * as css from './properties.css.ts';

const RAW=['Received: from b.example.org by c.example.org; Tue, 1 Jul 2026 09:00:05 +0000','Received: from a.example.org by b.example.org; Tue, 1 Jul 2026 09:00:00 +0000','From: Alice <alice@example.org>','Subject: Hello','X-Spam-Status: No, score=1.2 required=5.0','X-Spam-Score: 1.2','X-Priority: 1 (Highest)','','body'].join('\r\n');
const h=vi.hoisted(()=>({calls:[] as {name:string;args:Record<string,unknown>}[],destroyed:[] as string[],headers:'' as string,notDestroyed:false}));
vi.mock('../../api/transport.ts',()=>({createConfiguredClient:()=>({
 session:async()=>({capabilities:{},accounts:{},primaryAccounts:{},username:'me@example.org'}),
 jmap:vi.fn(async(request:JmapRequest):Promise<JmapResponse>=>{
  const [name,args,id]=request.methodCalls[0]!;const a=args as Record<string,unknown>;
  h.calls.push({name,args:a});
  if(name==='Email/get')return {methodResponses:[[name,{list:[{id:'m1',cc:[{name:'Carol',email:'carol@example.org'}],replyTo:[{email:'reply@example.org'}],messageId:['abc@example.org'],sentAt:'2026-07-01T08:59:00Z',size:4321,mailwomanPriority:'high',attachments:[{name:'a.pdf',type:'application/pdf',blobId:'m1.2',size:9000}]}],notFound:[]},id]],sessionState:'s'} as JmapResponse;
  if(name==='Email/location')return {methodResponses:[[name,{list:[{id:'m1',mailboxId:'inbox',mailboxPath:'INBOX',uid:42,uidValidity:7}],notFound:[]},id]],sessionState:'s'} as JmapResponse;
  if(name==='SecurityVerdict/get')return {methodResponses:[[name,{list:[{emailId:'m1',auth:{spf:{result:'pass',domain:'example.org'},dkim:{result:'pass',domain:'example.org',selector:'s'},dmarc:{result:'pass',policy:'reject',aligned:true},arc:{result:'none',chainLength:0}},plainLanguage:'Looks fine.',received:[{index:0,delayMs:5000,byHost:'b',fromHost:'a',protocol:null,timestamp:null,asn:null,asnOrg:null,country:null}],signature:null,encryption:{kind:'none',isEncrypted:false,decryptsClientSide:false},attachments:[{name:'a.pdf',declaredType:'application/pdf',detectedType:'application/pdf',mismatch:false,risk:'none'}],anomalies:['replyToMismatch']}]},id]],sessionState:'s'} as JmapResponse;
  if(name==='Email/set'){const ids=a['destroy'] as string[];if(h.notDestroyed)return {methodResponses:[[name,{notDestroyed:{[ids[0]!]:{type:'forbidden',description:'held'}}},id]],sessionState:'s'} as JmapResponse;h.destroyed.push(...ids);return {methodResponses:[[name,{destroyed:ids},id]],sessionState:'s'} as JmapResponse;}
  throw new Error(`unexpected ${name}`);
 }),
})}));
vi.mock('../../api/mailHeaders.ts',async()=>{const actual=await vi.importActual<typeof import('../../api/mailHeaders.ts')>('../../api/mailHeaders.ts');return {...actual,fetchMailHeaders:async()=>RAW.split(/\r?\n\r?\n/)[0]!};});

const json=(body:unknown)=>new Response(JSON.stringify(body),{status:200,headers:{'content-type':'application/json'}});
async function policy(values:Record<string,{value:unknown;locked?:boolean}>):Promise<void> {
 const settings:Record<string,unknown>={};
 for(const [key,entry] of Object.entries(values))settings[key]={source:'global',locked:false,...entry};
 await loadSettings({refresh:true,fetcher:(async()=>json({settings})) as Fetcher});
}
let closed=0;
async function open(over:Partial<Email>={},actions?:Parameters<typeof MessageDialogs>[0]['actions']) {
 const app=createAppState(makeClient());
 const email=mkEmail('m1',{subject:'Hello',from:[{name:'Alice',email:'alice@example.org'}],to:[{name:null,email:'me@example.org'}],blobId:'m1',mailboxIds:{inbox:true},keywords:{$seen:true,$flagged:true},mailwomanPriority:'high',...over});
 render(()=><AppContext.Provider value={app}><MessageDialogs email={email} {...(actions?{actions}:{})} onClose={()=>{closed++;}}/></AppContext.Provider>);
 await app.login({username:'me@example.org',password:'p'});
 return {app,dialog:await screen.findByRole('dialog')};
}
const tab=(name:string)=>screen.getByRole('tab',{name});
beforeEach(()=>{h.calls=[];h.destroyed=[];h.notDestroyed=false;closed=0;clearSettings();localStorage.clear();});
afterEach(()=>cleanup());

describe('message Properties dialog',()=>{
 it('is a labelled modal whose summary carries the address fields, priority and server location',async()=>{
  await policy({});
  const {dialog}=await open();
  expect(dialog).toHaveAttribute('aria-modal','true');
  expect(dialog.getAttribute('aria-labelledby')).toBeTruthy();
  expect(dialog.querySelector(`.${css.phone}`)).not.toBeNull();
  expect(screen.getAllByRole('tab').map(item=>item.textContent)).toEqual(['Summary','Scoring and security','Headers','Attachments','Actions']);
  const summary=screen.getByTestId('props-summary');
  await waitFor(()=>expect(summary).toHaveTextContent('carol@example.org'));
  expect(summary).toHaveTextContent('reply@example.org');
  expect(summary).toHaveTextContent('abc@example.org');
  expect(within(summary).getByTestId('priority-high')).toBeInTheDocument();
  await waitFor(()=>expect(summary).toHaveTextContent('UIDVALIDITY'));
  expect(summary).toHaveTextContent('42');
  expect(summary).toHaveTextContent('Go to folder');
 });
 it('hides the sections, raw headers and scoring the administrator turned off',async()=>{
  await policy({'properties.message.sections':{value:['summary','scoring','headers','actions']},'properties.headers.raw':{value:false},'properties.scoring.show':{value:false}});
  await open();
  expect(screen.getAllByRole('tab').map(item=>item.textContent)).toEqual(['Summary','Actions']);
 });
 it('masks the server location by policy',async()=>{
  await policy({'properties.remoteLocation.visibility':{value:'hidden'}});
  await open();
  const summary=screen.getByTestId('props-summary');
  await waitFor(()=>expect(h.calls.some(call=>call.name==='Email/location')).toBe(true));
  await waitFor(()=>expect(summary).toHaveTextContent('carol@example.org'));
  expect(summary).not.toHaveTextContent('UIDVALIDITY');
 });
 it('shows the spam verdict, authentication and signals',async()=>{
  await policy({});
  await open();
  fireEvent.click(tab('Scoring and security'));
  const panel=await screen.findByTestId('props-scoring');
  await waitFor(()=>expect(panel).toHaveTextContent('Looks fine.'));
  expect(panel).toHaveTextContent('SPF');
  expect(panel).toHaveTextContent('DMARC');
  expect(panel).toHaveTextContent('Reply-To address differs from the sender');
  expect(panel).toHaveTextContent('X-Spam-Status');
 });
 it('lists the headers, filters them and shows the Received chain with hop delays',async()=>{
  await policy({});
  await open();
  fireEvent.click(tab('Headers'));
  const panel=await screen.findByTestId('props-headers');
  await waitFor(()=>expect(panel).toHaveTextContent('X-Priority: 1'));
  fireEvent.input(within(panel).getByRole('searchbox'),{target:{value:'spam'}});
  expect(within(panel).getByLabelText('Raw message headers')).not.toHaveTextContent('X-Priority');
  fireEvent.input(within(panel).getByRole('searchbox'),{target:{value:''}});
  fireEvent.click(within(panel).getByRole('button',{name:/Received chain \(2\)/}));
  await waitFor(()=>expect(screen.getAllByTestId('props-hop-delay').some(node=>node.textContent==='5.0 s')).toBe(true));
 });
 it('lists the attachments with their risk',async()=>{
  await policy({});
  await open();
  fireEvent.click(tab('Attachments'));
  const panel=await screen.findByTestId('props-attachments');
  await waitFor(()=>expect(panel).toHaveTextContent('a.pdf'));
 });
 it('offers the downloads, or none when export.message.download.enabled is off',async()=>{
  await policy({});
  await open();
  fireEvent.click(tab('Actions'));
  await screen.findByTestId('props-message-actions');
  expect(screen.getByTestId('props-download-mbox')).toBeInTheDocument();
  expect(screen.getByRole('button',{name:'Copy message-id'})).toBeInTheDocument();
  cleanup();
  await policy({'export.message.download.enabled':{value:false}});
  await open();
  fireEvent.click(tab('Actions'));
  await screen.findByTestId('props-message-actions');
  expect(screen.queryByTestId('props-download-mbox')).toBeNull();
  expect(screen.queryByTestId('props-download-eml')).toBeNull();
 });
 it('permanent delete: one confirmation under "allow", saying there is no Undo',async()=>{
  await policy({'properties.message.permanentDelete':{value:'allow'}});
  await open();
  fireEvent.click(tab('Actions'));
  expect(screen.getByText(/There is no Undo/)).toBeInTheDocument();
  fireEvent.click(await screen.findByTestId('props-delete'));
  const confirm=await screen.findByRole('alertdialog');
  expect(confirm).toHaveTextContent('Undo is not available');
  expect(h.destroyed).toEqual([]);
  fireEvent.click(within(confirm).getByRole('button',{name:'Delete permanently'}));
  await waitFor(()=>expect(h.destroyed).toEqual(['m1']));
  await waitFor(()=>expect(closed).toBe(1));
 });
 it('permanent delete: "confirm-twice" asks a second time before it deletes',async()=>{
  await policy({'properties.message.permanentDelete':{value:'confirm-twice'}});
  await open();
  fireEvent.click(tab('Actions'));
  fireEvent.click(await screen.findByTestId('props-delete'));
  fireEvent.click(within(await screen.findByRole('alertdialog')).getByRole('button',{name:'Continue'}));
  const second=await screen.findByText('Really delete it for good?');
  expect(h.destroyed).toEqual([]);
  fireEvent.click(within(second.closest('[role="alertdialog"]') as HTMLElement).getByRole('button',{name:'Delete permanently'}));
  await waitFor(()=>expect(h.destroyed).toEqual(['m1']));
 });
 it('permanent delete: "deny" disables the button and explains why',async()=>{
  await policy({'properties.message.permanentDelete':{value:'deny',locked:true}});
  await open();
  fireEvent.click(tab('Actions'));
  const button=await screen.findByTestId('props-delete');
  expect(button).toBeDisabled();
  expect(button).toHaveAttribute('title',expect.stringContaining('does not allow permanent deletion'));
  fireEvent.click(button);
  expect(screen.queryByRole('alertdialog')).toBeNull();
  expect(h.destroyed).toEqual([]);
 });
 it('keeps the dialog open and says why when the server refuses the delete (a hold)',async()=>{
  await policy({});
  h.notDestroyed=true;
  await open();
  fireEvent.click(tab('Actions'));
  fireEvent.click(await screen.findByTestId('props-delete'));
  fireEvent.click(within(await screen.findByRole('alertdialog')).getByRole('button',{name:'Delete permanently'}));
  expect(await screen.findByText('held')).toBeInTheDocument();
  expect(closed).toBe(0);
 });
 it('runs a message action from the menu and closes first',async()=>{
  await policy({});
  const onSpam=vi.fn();
  await open({},{onSpam});
  fireEvent.click(tab('Actions'));
  fireEvent.click(await screen.findByRole('button',{name:'Mark as spam'}));
  expect(closed).toBe(1);expect(onSpam).toHaveBeenCalledOnce();
 });
});
