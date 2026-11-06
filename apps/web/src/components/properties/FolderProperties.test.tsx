import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor,within} from '@solidjs/testing-library';
import {makeClient,MAILBOXES} from '../appHarness.tsx';
import {createAppState} from '../../state/store.ts';
import {AppContext} from '../../state/context.ts';
import {clearSettings,loadSettings,type Fetcher} from '../../api/settings.ts';
import {forgetFolderStats} from './folderStats.ts';
import type {JmapRequest,JmapResponse,Mailbox} from '../../api/jmap-types.ts';
import {FolderPropertiesDialog} from './FolderProperties.tsx';
import * as css from './properties.css.ts';

const h=vi.hoisted(()=>({
 stats:{} as Record<string,unknown>,
 prefs:{} as Record<string,unknown>,
 sets:[] as Record<string,unknown>[],
 estimate:{} as Record<string,unknown>,
 http:[] as string[],
}));
vi.mock('../../api/transport.ts',()=>({createConfiguredClient:()=>({
 session:async()=>({capabilities:{},accounts:{},primaryAccounts:{},username:'me@example.org'}),
 jmap:vi.fn(async(request:JmapRequest):Promise<JmapResponse>=>{
  const [name,args,id]=request.methodCalls[0]!;
  if(name==='Mailbox/stats')return {methodResponses:[[name,{...h.stats,mailboxId:(args as {mailboxId:string}).mailboxId},id]],sessionState:'s'} as JmapResponse;
  throw new Error(`unexpected ${name}`);
 }),
})}));
vi.mock('../../api/http.ts',()=>({apiFetch:vi.fn(async(url:string)=>{
 h.http.push(url);
 return new Response(JSON.stringify(h.estimate),{status:typeof h.estimate['status']==='number'?h.estimate['status'] as number:200,headers:{'content-type':'application/json'}});
})}));
vi.mock('../../api/accountPreferences.ts',async()=>{
 const actual=await vi.importActual<typeof import('../../api/accountPreferences.ts')>('../../api/accountPreferences.ts');
 return {...actual,accountUiPreferences:()=>h.prefs,setAccountUiPreferences:(patch:Record<string,unknown>)=>{h.sets.push(patch);Object.assign(h.prefs,patch);}};
});

const json=(body:unknown)=>new Response(JSON.stringify(body),{status:200,headers:{'content-type':'application/json'}});
async function policy(values:Record<string,{value:unknown;locked?:boolean;source?:string}>):Promise<void> {
 const settings:Record<string,unknown>={};
 for(const [key,entry] of Object.entries(values))settings[key]={source:'user',locked:false,...entry};
 await loadSettings({refresh:true,fetcher:(async()=>json({settings})) as Fetcher});
}
const STATS={role:null,totalEmails:3,unreadEmails:1,cachedEmails:3,approximate:false,sizeBytes:2048,flaggedEmails:1,draftEmails:0,oldest:'2026-01-02T03:04:05Z',newest:'2026-02-03T03:04:05Z',bodiesCached:2,largest:[{id:'m1',size:1500}],
 remote:{path:'Archive',upstreamName:'Archive',delimiter:'/',uidValidity:77,uidNext:9,highestModSeq:5,namespace:'personal',host:'imap.example.org'}};
let closed=0;
async function open(box:Mailbox=MAILBOXES[1]!,actions?:Parameters<typeof FolderPropertiesDialog>[0]['actions']) {
 const app=createAppState(makeClient());
 render(()=><AppContext.Provider value={app}><FolderPropertiesDialog box={box} {...(actions?{actions}:{})} onClose={()=>{closed++;}}/></AppContext.Provider>);
 await app.login({username:'me@example.org',password:'p'});
 return {app,dialog:await screen.findByRole('dialog')};
}
const tab=(name:string)=>screen.getByRole('tab',{name});

beforeEach(()=>{h.stats={...STATS};h.prefs={};h.sets=[];h.estimate={folders:1,messages:3,bytes:2048,maxBytes:1e9,maxMessages:1000,format:'eml',formats:['eml','mbox'],subfolders:true,allowed:true};h.http=[];closed=0;forgetFolderStats();clearSettings();localStorage.clear();});
afterEach(()=>cleanup());

describe('folder Properties dialog',()=>{
 it('is a labelled modal with the sections in order and the phone layout class',async()=>{
  await policy({});
  const {dialog}=await open();
  expect(dialog).toHaveAttribute('aria-modal','true');
  expect(dialog.getAttribute('aria-label')??dialog.textContent).toMatch(/Properties: .?Archive/);
  expect(dialog.getAttribute('aria-labelledby')).toBeTruthy();
  expect(screen.getAllByRole('tab').map(item=>item.textContent)).toEqual(['General','Remote location','Size and counts','Offline cache','Overrides','Notes','Actions']);
  expect(screen.getByRole('tabpanel')).toBeInTheDocument();
  expect(dialog.querySelector(`.${css.phone}`)).not.toBeNull();
  expect(within(dialog).getByTestId('props-general')).toHaveTextContent('Type');
 });
 it('moves between tabs with the arrow keys',async()=>{
  await policy({});
  await open();
  tab('General').focus();
  fireEvent.keyDown(tab('General'),{key:'ArrowRight'});
  expect(tab('Remote location')).toHaveAttribute('aria-selected','true');
  fireEvent.keyDown(tab('Remote location'),{key:'End'});
  expect(tab('Actions')).toHaveAttribute('aria-selected','true');
 });
 it('hides sections the administrator did not list, and notes and overrides when switched off',async()=>{
  await policy({'properties.folder.sections':{value:['general','notes','overrides','actions'],locked:true,source:'global'},'properties.folder.notes.enabled':{value:false,source:'global',locked:true},'properties.folder.overrides.allowed':{value:[],source:'global'}});
  await open();
  expect(screen.getAllByRole('tab').map(item=>item.textContent)).toEqual(['General','Actions']);
 });
 it('masks the remote location by policy',async()=>{
  await policy({'properties.remoteLocation.visibility':{value:'masked-host',source:'global'}});
  await open();
  fireEvent.click(tab('Remote location'));
  const panel=await screen.findByTestId('props-remote');
  await waitFor(()=>expect(panel).toHaveTextContent('Archive'));
  expect(panel).toHaveTextContent('UIDVALIDITY');
  expect(panel).toHaveTextContent('i***.example.org');
  expect(panel).not.toHaveTextContent('imap.example.org');
  cleanup();forgetFolderStats();
  await policy({'properties.remoteLocation.visibility':{value:'path-only',source:'global'}});
  await open();fireEvent.click(tab('Remote location'));
  const pathOnly=await screen.findByTestId('props-remote');
  await waitFor(()=>expect(pathOnly).toHaveTextContent('Archive'));
  expect(pathOnly).not.toHaveTextContent('example.org');
  cleanup();
  await policy({'properties.remoteLocation.visibility':{value:'hidden',source:'global'}});
  await open();fireEvent.click(tab('Remote location'));
  expect(await screen.findByText(/does not show the server location/)).toBeInTheDocument();
  expect(screen.queryByText('UIDVALIDITY')).toBeNull();
 });
 it('shows size and counts and says when they are approximate',async()=>{
  await policy({});
  h.stats={...STATS,approximate:true,cachedEmails:2};
  await open();
  fireEvent.click(tab('Size and counts'));
  const panel=await screen.findByTestId('props-size');
  await waitFor(()=>expect(panel).toHaveTextContent('About 2.0 KB (approximate)'));
  expect(panel).toHaveTextContent('Approximate: 2 of 3 messages are synced');
  expect(panel).toHaveTextContent('Flagged');
 });
 it('does not compute the size when the policy says off, until asked',async()=>{
  await policy({'properties.folder.sizeCompute':{value:'off',source:'global'}});
  await open();
  fireEvent.click(tab('Size and counts'));
  expect(await screen.findByText(/not worked out automatically/)).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button',{name:'Work out the size'}));
  await waitFor(()=>expect(screen.getByTestId('props-size')).toHaveTextContent('2.0 KB'));
 });
 it('shows the effective override with its source, and writes and clears an override',async()=>{
  await policy({'mail.folderCounts':{value:'both',source:'user'}});
  await open();
  fireEvent.click(tab('Overrides'));
  const row=await screen.findByTestId('ovr-counters');
  expect(row).toHaveAttribute('data-source','user');
  expect(row).toHaveTextContent('Now: Unread and total (your setting)');
  fireEvent.change(within(row).getByRole('combobox'),{target:{value:'total'}});
  expect(h.sets.at(-1)).toEqual({folderProps:{[MAILBOXES[1]!.id]:{counters:'total'}}});
  fireEvent.change(within(screen.getByTestId('ovr-counters')).getByRole('combobox'),{target:{value:''}});
  expect(h.sets.at(-1)).toEqual({folderProps:{}});
 });
 it('shows an administrator default and an administrator lock, and offers no control for a locked one',async()=>{
  await policy({'mail.folderCounts':{value:'unread',source:'global',locked:true},'mail.compactFolderBadges':{value:false,source:'domain'}});
  await open();
  fireEvent.click(tab('Overrides'));
  const locked=await screen.findByTestId('ovr-counters');
  expect(locked).toHaveAttribute('data-source','admin-locked');
  expect(within(locked).queryByRole('combobox')).toBeNull();
  expect(within(locked).getByTestId('ovr-counters-locked')).toHaveAttribute('title',expect.stringContaining('locked this setting'));
  const dflt=screen.getByTestId('ovr-compactBadge');
  expect(dflt).toHaveAttribute('data-source','admin-default');
  expect(dflt).toHaveTextContent('administrator default');
 });
 it('lists only the overrides the administrator allows',async()=>{
  await policy({'properties.folder.overrides.allowed':{value:['counters','sort'],source:'global'}});
  await open();
  fireEvent.click(tab('Overrides'));
  await screen.findByTestId('ovr-counters');
  expect(screen.getByTestId('ovr-sort')).toBeInTheDocument();
  expect(screen.queryByTestId('ovr-threading')).toBeNull();
 });
 it('saves a note with its length shown, bounded by the administrator maximum',async()=>{
  await policy({'properties.folder.notes.maxLength':{value:120,source:'global'}});
  await open();
  fireEvent.click(tab('Notes'));
  const box=await screen.findByRole('textbox',{name:/notes for this folder/});
  expect(box).toHaveAttribute('maxlength','120');
  expect(screen.getByRole('button',{name:'Save note'})).toBeDisabled();
  fireEvent.input(box,{target:{value:'Quarterly invoices'}});
  expect(screen.getByText('18 of 120 characters')).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button',{name:'Save note'}));
  expect(h.sets.at(-1)).toEqual({folderProps:{[MAILBOXES[1]!.id]:{notes:'Quarterly invoices'}}});
 });
 it('clears the offline cache only after a confirmation',async()=>{
  await policy({});
  const {app}=await open();
  const clear=vi.spyOn(app,'clearFolderCache');const whole=vi.spyOn(app,'clearMailCache');
  fireEvent.click(tab('Offline cache'));
  fireEvent.click(await screen.findByTestId('props-clear-cache'));
  const confirm=await screen.findByRole('alertdialog');
  expect(clear).not.toHaveBeenCalled();
  fireEvent.click(within(confirm).getByRole('button',{name:'Clear offline cache'}));
  await waitFor(()=>expect(clear).toHaveBeenCalledOnce());
  expect(clear).toHaveBeenCalledWith(MAILBOXES[1]!.id);
  expect(whole).not.toHaveBeenCalled();
 });
 it('hides the clear button when the administrator turned it off',async()=>{
  await policy({'properties.offlineClear.enabled':{value:false,source:'global'}});
  await open();
  fireEvent.click(tab('Offline cache'));
  await screen.findByTestId('props-offline');
  expect(screen.queryByTestId('props-clear-cache')).toBeNull();
 });
 it('checks the ZIP size first and only then offers the download',async()=>{
  await policy({});
  await open();
  fireEvent.click(tab('Actions'));
  const start=await screen.findByTestId('props-zip-start');
  expect(start).toBeDisabled();
  fireEvent.click(screen.getByTestId('props-zip-check'));
  expect(await screen.findByTestId('props-zip-estimate')).toHaveTextContent('3 messages, 2.0 KB.');
  const call=h.http.find(url=>url.includes('/api/export/folder/'));
  expect(call).toContain('estimate=1');
  expect(start).toBeEnabled();
 });
 it('refuses a ZIP over the administrator cap and keeps the button off',async()=>{
  await policy({});
  h.estimate={...h.estimate,bytes:5e9,maxBytes:2e9,allowed:false,reason:'exportTooLarge'};
  await open();
  fireEvent.click(tab('Actions'));
  fireEvent.click(await screen.findByTestId('props-zip-check'));
  expect(await screen.findByTestId('props-zip-estimate')).toHaveTextContent('over the limit of 1.9 GB');
  expect(screen.getByTestId('props-zip-start')).toBeDisabled();
 });
 it('shows the server refusal when folder download is off',async()=>{
  await policy({});
  h.estimate={status:403,error:'exportDisabled'};
  await open();
  fireEvent.click(tab('Actions'));
  fireEvent.click(await screen.findByTestId('props-zip-check'));
  expect(await screen.findByTestId('props-zip-problem')).toHaveTextContent('turned off by your administrator');
 });
 it('offers no ZIP section when export.folder.enabled is off',async()=>{
  await policy({'export.folder.enabled':{value:false,source:'global'}});
  await open();
  fireEvent.click(tab('Actions'));
  await screen.findByTestId('props-actions');
  expect(screen.queryByTestId('props-zip-start')).toBeNull();
  expect(screen.getByRole('button',{name:'Copy folder path'})).toBeInTheDocument();
 });
 it('runs a folder action from the menu and closes first',async()=>{
  await policy({});
  const onEmpty=vi.fn();
  await open(MAILBOXES[1]!,{onEmpty});
  fireEvent.click(tab('Actions'));
  fireEvent.click(await screen.findByRole('button',{name:'Empty folder'}));
  expect(closed).toBe(1);expect(onEmpty).toHaveBeenCalledOnce();
 });
 it('closes on Escape',async()=>{
  await policy({});
  const {dialog}=await open();
  fireEvent.keyDown(dialog,{key:'Escape'});
  await waitFor(()=>expect(closed).toBe(1));
 });
});
