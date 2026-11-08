import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor,within} from '@solidjs/testing-library';
import {makeClient,MAILBOXES} from '../appHarness.tsx';
import {createAppState} from '../../state/store.ts';
import {AppContext} from '../../state/context.ts';
import {clearSettings,loadSettings,type Fetcher} from '../../api/settings.ts';
import {forgetFolderStats} from './folderStats.ts';
import type {JmapRequest,JmapResponse,Mailbox} from '../../api/jmap-types.ts';
import {FolderPropertiesDialog} from './FolderProperties.tsx';
import * as css from './folderDialog.css.ts';

const h=vi.hoisted(()=>({
 stats:{} as Record<string,unknown>,
 prefs:{} as Record<string,unknown>,
 sets:[] as Record<string,unknown>[],
 estimate:{} as Record<string,unknown>,
 http:[] as string[],
 fail:false,
 hang:false,
}));
vi.mock('../../api/transport.ts',()=>({createConfiguredClient:()=>({
 session:async()=>({capabilities:{},accounts:{},primaryAccounts:{},username:'me@example.org'}),
 jmap:vi.fn(async(request:JmapRequest):Promise<JmapResponse>=>{
  const [name,args,id]=request.methodCalls[0]!;
  if(name==='Mailbox/stats'&&h.hang)return await new Promise<JmapResponse>(()=>{});
  if(name==='Mailbox/stats'&&h.fail)return {methodResponses:[['error',{type:'serverFail'},id]],sessionState:'s'} as unknown as JmapResponse;
  if(name==='Mailbox/stats'&&h.hang)return await new Promise<JmapResponse>(()=>{});
  if(name==='Mailbox/stats'&&h.fail)return {methodResponses:[['error',{type:'serverFail'},id]],sessionState:'s'} as unknown as JmapResponse;
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
 const solid=await vi.importActual<typeof import('solid-js')>('solid-js');
 const [tick,bump]=solid.createSignal(0);
 return {...actual,accountUiPreferences:()=>{tick();return h.prefs;},setAccountUiPreferences:(patch:Record<string,unknown>)=>{h.sets.push(patch);h.prefs={...h.prefs,...patch};bump(value=>value+1);}};
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

beforeEach(()=>{h.stats={...STATS};h.prefs={};h.sets=[];h.estimate={folders:1,messages:3,bytes:2048,maxBytes:1e9,maxMessages:1000,format:'eml',formats:['eml','mbox'],subfolders:true,allowed:true};h.http=[];h.fail=false;h.hang=false;closed=0;forgetFolderStats();clearSettings();localStorage.clear();});
afterEach(()=>cleanup());

describe('folder Properties dialog',()=>{
 it('is a labelled modal with the sections in order and the phone layout class',async()=>{
  await policy({});
  const {dialog}=await open();
  expect(dialog).toHaveAttribute('aria-modal','true');
  expect(dialog.getAttribute('aria-label')??dialog.textContent).toMatch(/Properties: .?Archive/);
  expect(dialog.getAttribute('aria-labelledby')).toBeTruthy();
  expect(screen.getAllByRole('tab').map(item=>item.textContent)).toEqual(['General','Appearance','Remote location','Size and storage','Offline cache','Display','Notes','Actions']);
  expect(screen.getByRole('tabpanel')).toBeInTheDocument();
  expect(dialog).toHaveClass(css.phone);
  expect(within(dialog).getByTestId('props-general')).toHaveTextContent('Type');
 });
 it('moves between tabs with the arrow keys',async()=>{
  await policy({});
  await open();
  tab('General').focus();
  fireEvent.keyDown(tab('General'),{key:'ArrowRight'});
  expect(tab('Appearance')).toHaveAttribute('aria-selected','true');
  fireEvent.keyDown(tab('Appearance'),{key:'End'});
  expect(tab('Actions')).toHaveAttribute('aria-selected','true');
 });
 it('hides sections the administrator did not list, and notes and overrides when switched off',async()=>{
  await policy({'properties.folder.sections':{value:['general','notes','overrides','actions'],locked:true,source:'global'},'properties.folder.notes.enabled':{value:false,source:'global',locked:true},'properties.folder.overrides.allowed':{value:[],source:'global'}});
  await open();
  expect(screen.getAllByRole('tab').map(item=>item.textContent)).toEqual(['General','Appearance','Actions']);
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
  fireEvent.click(tab('Size and storage'));
  const panel=await screen.findByTestId('props-size');
  await waitFor(()=>expect(panel).toHaveTextContent('About 2.0 KB (approximate)'));
  expect(panel).toHaveTextContent('Approximate: 2 of 3 messages are synced');
  expect(panel).toHaveTextContent('Flagged');
 });
 it('does not compute the size when the policy says off, until asked',async()=>{
  await policy({'properties.folder.sizeCompute':{value:'off',source:'global'}});
  await open();
  fireEvent.click(tab('Size and storage'));
  expect(await screen.findByText(/not worked out automatically/)).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button',{name:'Work out the size'}));
  await waitFor(()=>expect(screen.getByTestId('props-size')).toHaveTextContent('2.0 KB'));
 });
 it('shows the default in force as a chip, and writes and resets an override',async()=>{
  await policy({'mail.folderCounts':{value:'both',source:'user'}});
  await open();
  fireEvent.click(tab('Display'));
  const row=await screen.findByTestId('ovr-counters');
  expect(row).toHaveAttribute('data-source','user');
  expect(within(row).getByTestId('ovr-counters-source')).toHaveTextContent('Using your default (Unread and total)');
  expect(within(row).queryByTestId('ovr-counters-reset')).toBeNull();
  fireEvent.click(within(row).getByRole('radio',{name:'Total'}));
  expect(h.sets.at(-1)).toEqual({folderProps:{[MAILBOXES[1]!.id]:{counters:'total'}}});
  const after=screen.getByTestId('ovr-counters');
  expect(within(after).getByTestId('ovr-counters-source')).toHaveTextContent('Overridden for this folder (Total)');
  expect(within(after).getByRole('radio',{name:'Total'})).toHaveAttribute('aria-checked','true');
  fireEvent.click(within(after).getByTestId('ovr-counters-reset'));
  expect(h.sets.at(-1)).toEqual({folderProps:{}});
  expect(within(screen.getByTestId('ovr-counters')).getByTestId('ovr-counters-source')).toHaveTextContent('Using your default');
 });
 it('previews the badge with the choices as the sidebar will draw it',async()=>{
  await policy({'mail.folderCounts':{value:'both',source:'user'}});
  await open(MAILBOXES[1]!);
  fireEvent.click(tab('Display'));
  const preview=await screen.findByTestId('ovr-preview');
  expect(within(preview).getByTestId('ovr-preview-badge')).toBeInTheDocument();
  fireEvent.click(within(screen.getByTestId('ovr-counters')).getByRole('radio',{name:'None'}));
  expect(within(screen.getByTestId('ovr-preview')).getByTestId('ovr-preview-none')).toBeInTheDocument();
 });
 it('labels the overrides the app does not apply yet and keeps them behind a disclosure',async()=>{
  await policy({});
  await open();
  fireEvent.click(tab('Display'));
  const more=await screen.findByTestId('props-ovr-more');
  expect(more.tagName).toBe('DETAILS');
  expect(more).not.toHaveAttribute('open');
  expect(more).toHaveTextContent('More options (not applied yet)');
  expect(within(screen.getByTestId('ovr-sort')).getByTestId('ovr-sort-later')).toHaveTextContent('Saved for later');
  expect(within(screen.getByTestId('ovr-counters')).queryByTestId('ovr-counters-later')).toBeNull();
 });
 it('shows an administrator default and an administrator lock, and offers no control for a locked one',async()=>{
  await policy({'mail.folderCounts':{value:'unread',source:'global',locked:true},'mail.compactFolderBadges':{value:false,source:'domain'}});
  await open();
  fireEvent.click(tab('Display'));
  const locked=await screen.findByTestId('ovr-counters');
  expect(locked).toHaveAttribute('data-source','admin-locked');
  expect(within(locked).queryByRole('radio')).toBeNull();
  expect(within(locked).getByTestId('ovr-counters-locked')).toHaveAttribute('title',expect.stringContaining('locked this setting'));
  expect(within(locked).getByTestId('ovr-counters-source')).toHaveTextContent('Locked by your administrator (Unread)');
  const dflt=screen.getByTestId('ovr-compactBadge');
  expect(dflt).toHaveAttribute('data-source','admin-default');
  expect(dflt).toHaveTextContent('Using the administrator default (Off)');
 });
 it('lists only the overrides the administrator allows',async()=>{
  await policy({'properties.folder.overrides.allowed':{value:['counters','sort'],source:'global'}});
  await open();
  fireEvent.click(tab('Display'));
  await screen.findByTestId('ovr-counters');
  expect(screen.getByTestId('ovr-sort')).toBeInTheDocument();
  expect(screen.queryByTestId('ovr-threading')).toBeNull();
 });
 it('offers empty-folder visibility as an applied choice with inherited default, explicit false and reset',async()=>{
  h.prefs={folderProps:{[MAILBOXES[1]!.id]:{notes:'keep this note'}}};
  await policy({});await open();fireEvent.click(tab('Display'));
  const card=await screen.findByTestId('props-folder-visibility');
  const row=within(card).getByTestId('ovr-hideWhenEmpty');
  expect(row.closest('details')).toBeNull();
  expect(within(row).queryByTestId('ovr-hideWhenEmpty-later')).toBeNull();
  expect(row).toHaveAttribute('data-source','user');
  expect(within(row).getByTestId('ovr-hideWhenEmpty-source')).toHaveTextContent('Using your default (Off)');
  expect(within(row).getByRole('radio',{name:'Same as default'})).toHaveAttribute('aria-checked','true');
  fireEvent.click(within(row).getByRole('radio',{name:'On'}));
  expect(h.sets.at(-1)).toEqual({folderProps:{[MAILBOXES[1]!.id]:{notes:'keep this note',hideWhenEmpty:true}}});
  expect(row).toHaveAttribute('data-source','folder');
  fireEvent.click(within(row).getByRole('radio',{name:'Off'}));
  expect(h.sets.at(-1)).toEqual({folderProps:{[MAILBOXES[1]!.id]:{notes:'keep this note',hideWhenEmpty:false}}});
  expect(within(row).getByRole('radio',{name:'Off'})).toHaveAttribute('aria-checked','true');
  fireEvent.click(within(row).getByTestId('ovr-hideWhenEmpty-reset'));
  expect(h.sets.at(-1)).toEqual({folderProps:{[MAILBOXES[1]!.id]:{notes:'keep this note'}}});
  expect(within(row).getByRole('radio',{name:'Same as default'})).toHaveAttribute('aria-checked','true');
 });
 it('restores inherited administrator visibility when Same as default is selected',async()=>{
  await policy({'folders.hideEmpty':{value:true,source:'global'}});
  h.prefs={folderProps:{[MAILBOXES[1]!.id]:{hideWhenEmpty:false}}};
  await open();fireEvent.click(tab('Display'));const row=await screen.findByTestId('ovr-hideWhenEmpty');
  expect(within(row).getByTestId('ovr-hideWhenEmpty-source')).toHaveTextContent('Overridden for this folder (Off)');
  fireEvent.click(within(row).getByRole('radio',{name:'Same as default'}));
  expect(h.sets.at(-1)).toEqual({folderProps:{}});
  expect(row).toHaveAttribute('data-source','admin-default');
  expect(within(row).getByTestId('ovr-hideWhenEmpty-source')).toHaveTextContent('Using the administrator default (On)');
 });
 it('shows locked visibility without editable or reset controls even over an existing folder override',async()=>{
  await policy({'folders.hideEmpty':{value:true,source:'global',locked:true}});
  h.prefs={folderProps:{[MAILBOXES[1]!.id]:{hideWhenEmpty:false}}};
  await open();fireEvent.click(tab('Display'));const row=await screen.findByTestId('ovr-hideWhenEmpty');
  expect(row).toHaveAttribute('data-source','admin-locked');
  expect(within(row).getByTestId('ovr-hideWhenEmpty-source')).toHaveTextContent('Locked by your administrator (On)');
  expect(within(row).queryByRole('radio')).toBeNull();
  expect(within(row).queryByTestId('ovr-hideWhenEmpty-reset')).toBeNull();
  expect(h.sets).toEqual([]);
 });
 it('respects the visibility override allowlist independently of the counter controls',async()=>{
  await policy({'properties.folder.overrides.allowed':{value:['hideWhenEmpty'],source:'global'}});
  await open();fireEvent.click(tab('Display'));
  expect(await screen.findByTestId('ovr-hideWhenEmpty')).toBeInTheDocument();
  expect(screen.queryByTestId('ovr-counters')).toBeNull();
  await policy({'properties.folder.overrides.allowed':{value:['counters'],source:'global'}});
  expect(screen.queryByTestId('props-folder-visibility')).toBeNull();
  expect(screen.getByTestId('ovr-counters')).toBeInTheDocument();
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
  expect(screen.getByTestId('props-notes-saved')).toHaveTextContent('Saved');
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
 it('saves the note when the field loses focus, and clears the Saved flag on the next edit',async()=>{
  await policy({});
  await open();
  fireEvent.click(tab('Notes'));
  const box=await screen.findByRole('textbox',{name:/notes for this folder/});
  fireEvent.input(box,{target:{value:'Remember'}});
  expect(h.sets).toHaveLength(0);
  fireEvent.blur(box);
  expect(h.sets.at(-1)).toEqual({folderProps:{[MAILBOXES[1]!.id]:{notes:'Remember'}}});
  expect(screen.getByTestId('props-notes-saved')).toHaveTextContent('Saved');
  fireEvent.input(box,{target:{value:'Remember more'}});
  expect(screen.getByTestId('props-notes-saved')).toHaveTextContent('');
 });
 it('shows the header: name, path with a copy button, type badge, and the switches write to the preferences',async()=>{
  await policy({});
  const writeText=vi.fn(async()=>{});Object.defineProperty(navigator,'clipboard',{value:{writeText},configurable:true});
  await open();
  expect(screen.getByRole('heading',{name:/Archive/})).toBeInTheDocument();
  expect(screen.getByTestId('props-type-badge')).toHaveTextContent('Archive');
  expect(screen.getByTestId('props-breadcrumb')).toHaveTextContent('Archive');
  fireEvent.click(screen.getByTestId('props-copy-path'));
  await waitFor(()=>expect(writeText).toHaveBeenCalledWith('Archive'));
  const star=screen.getByTestId('props-starred');
  expect(star).toHaveAttribute('aria-pressed','false');
  fireEvent.click(star);
  expect(h.sets.at(-1)).toEqual({folderStarred:[MAILBOXES[1]!.id]});
  expect(screen.getByTestId('props-pinned')).toHaveAttribute('aria-pressed','false');
  fireEvent.click(screen.getByTestId('props-pinned'));
  expect(h.sets.at(-1)).toEqual({folderPinned:[MAILBOXES[1]!.id]});
 });
 it('renames the folder in place from the header, ending with Escape',async()=>{
  await policy({});
  const box={...MAILBOXES[1]!,role:null,myRights:{mayRename:true}} as unknown as Mailbox;
  const app=createAppState(makeClient({mailboxes:[box]} as never));
  await app.login({username:'me@example.org',password:'p'});
  render(()=><AppContext.Provider value={app}><FolderPropertiesDialog box={box} onClose={()=>{}}/></AppContext.Provider>);
  const pencil=await screen.findByTestId('props-rename',undefined,{timeout:3000});
  fireEvent.click(pencil);
  const field=await screen.findByRole('textbox',{name:/Archive/});
  expect(field).toHaveFocus();
  fireEvent.keyDown(field,{key:'Escape'});
  await waitFor(()=>expect(screen.getByRole('heading',{name:/Archive/})).toBeInTheDocument());
 });
 it('draws skeletons only inside the tiles while the numbers are computed',async()=>{
  await policy({});
  h.hang=true;
  await open();
  expect(screen.getByTestId('tile-total')).toHaveTextContent('0');
  expect(within(screen.getByTestId('tile-size')).getByTestId('props-skeleton')).toBeInTheDocument();
  expect(within(screen.getByTestId('tile-newest')).getByTestId('props-skeleton')).toBeInTheDocument();
  expect(screen.getByRole('tabpanel')).toBeInTheDocument();
  expect(screen.getByTestId('props-general')).toBeInTheDocument();
 });
 it('shows the tile numbers once computed',async()=>{
  await policy({});
  await open();
  await waitFor(()=>expect(screen.getByTestId('tile-size')).toHaveTextContent('2.0 KB'));
  expect(screen.getByTestId('tile-total')).toHaveTextContent('3');
  expect(screen.getByTestId('tile-unread')).toHaveTextContent('1');
 });
 it('says in the tile when the size failed and offers a retry that works',async()=>{
  await policy({});
  h.fail=true;
  await open();
  const retry=await screen.findAllByTestId('props-tile-retry');
  expect(retry.length).toBeGreaterThan(0);
  h.fail=false;
  fireEvent.click(retry[0]!);
  await waitFor(()=>expect(screen.getByTestId('tile-size')).toHaveTextContent('2.0 KB'));
 });
 it('offers a Calculate size button in the tile when the policy leaves it to the user',async()=>{
  await policy({'properties.folder.sizeCompute':{value:'off',source:'global'}});
  await open();
  const button=within(screen.getByTestId('tile-size')).getByTestId('props-calc-size');
  fireEvent.click(button);
  await waitFor(()=>expect(screen.getByTestId('tile-size')).toHaveTextContent('2.0 KB'));
 });
 it('remembers the last tab for the next time the dialog opens',async()=>{
  await policy({});
  await open();
  fireEvent.click(tab('Notes'));
  cleanup();
  await open();
  expect(tab('Notes')).toHaveAttribute('aria-selected','true');
 });
 it('falls back to the first tab when the remembered one is no longer offered',async()=>{
  await policy({'properties.folder.sections':{value:['general','actions'],source:'global'}});
  localStorage.setItem('mw.folderProps.tab:acct1','notes');
  await open();
  expect(tab('General')).toHaveAttribute('aria-selected','true');
 });
 it('offers only the actions the menu supplied, and a separate danger zone',async()=>{
  await policy({});
  await open();
  fireEvent.click(tab('Actions'));
  await screen.findByTestId('props-actions');
  expect(screen.queryByTestId('props-danger')).toBeNull();
  expect(screen.queryByRole('button',{name:'Mark all read'})).toBeNull();
  expect(screen.queryByRole('button',{name:'Move to…'})).toBeNull();
  cleanup();
  const onMove=vi.fn(),onMarkRead=vi.fn(),onDelete=vi.fn();
  await open(MAILBOXES[1]!,{onMove,onMarkRead,onDelete,onEmpty:()=>{}});
  fireEvent.click(tab('Actions'));
  const danger=await screen.findByTestId('props-danger');
  expect(within(danger).getByRole('button',{name:'Delete folder'})).toBeInTheDocument();
  expect(within(danger).getByRole('button',{name:'Empty folder'})).toBeInTheDocument();
  fireEvent.click(screen.getByRole('button',{name:'Move to…'}));
  expect(closed).toBe(1);expect(onMove).toHaveBeenCalledOnce();
 });
 it('previews the folder row in Appearance and resets icon and colour to the default',async()=>{
  await policy({});
  h.prefs={folderIcons:{[MAILBOXES[1]!.id]:'star'},folderProps:{[MAILBOXES[1]!.id]:{badgeColour:'#3b82f6'}}};
  await open();
  fireEvent.click(tab('Appearance'));
  const preview=await screen.findByTestId('props-preview');
  expect(preview).toHaveTextContent('Archive');
  expect(screen.getByTestId('props-swatches')).toBeInTheDocument();
  fireEvent.click(screen.getByTestId('props-appearance-reset'));
  expect(h.sets.some(patch=>'folderIcons' in patch&&Object.keys(patch['folderIcons'] as object).length===0)).toBe(true);
 });
 it('exposes named landmarks for assistive technology: dialog, tablist, tabpanel, labelled meters',async()=>{
  await policy({});
  const {dialog}=await open();
  expect(dialog).toHaveAttribute('aria-labelledby');
  expect(screen.getByRole('tablist')).toHaveAttribute('aria-label','Sections');
  expect(screen.getByRole('tabpanel')).toHaveAttribute('aria-labelledby',tab('General').id);
  fireEvent.click(tab('Offline cache'));
  expect(await screen.findByRole('meter',{name:'Stored on the server for fast opening'})).toBeInTheDocument();
 });
 it('closes on Escape',async()=>{
  await policy({});
  const {dialog}=await open();
  fireEvent.keyDown(dialog,{key:'Escape'});
  await waitFor(()=>expect(closed).toBe(1));
 });
});
