import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@solidjs/testing-library';
import axe from 'axe-core';
import type {Email} from '../../api/jmap-types.ts';
import {clearSettings,loadSettings,type Fetcher} from '../../api/settings.ts';
import {mkEmail} from '../appHarness.tsx';

const h=vi.hoisted(()=>({calls:[] as string[],status:200,body:'RAW' as string,error:'' as string,gate:null as null|Promise<void>,saved:[] as string[]}));
vi.mock('../../api/http.ts',async importOriginal=>({...await importOriginal<typeof import('../../api/http.ts')>(),apiFetch:vi.fn(async(url:string,init?:RequestInit)=>{
  h.calls.push(url);
  if(h.gate)await h.gate;
  if(init?.signal?.aborted)throw new DOMException('cancelled','AbortError');
  if(url.includes('estimate=1'))return new Response(JSON.stringify({folders:1,messages:12,bytes:3000,maxBytes:2000,maxMessages:100,format:'eml',formats:['eml','mbox'],subfolders:false,allowed:h.error==='',...(h.error?{reason:h.error}:{})}),{status:200});
  if(h.status!==200)return new Response(JSON.stringify({error:h.error}),{status:h.status});
  return new Response(h.body,{status:200});
})}));
vi.mock('../properties/folderExport.ts',async()=>{
  const actual=await vi.importActual<typeof import('../properties/folderExport.ts')>('../properties/folderExport.ts');
  return {...actual,saveBlob:(_blob:Blob,name:string)=>{h.saved.push(name);}};
});
import {ExportDialog,problemKey} from './ExportDialog.tsx';
import {ExportError} from './exportRun.ts';
import {installLinkedMailbox} from '../../api/linkedMailboxSession.ts';
import type {ScopeSource} from './exportPlan.ts';

async function policy(values:Record<string,unknown>):Promise<void> {
  const settings=Object.fromEntries(Object.entries(values).map(([key,value])=>[key,{value,source:'global',locked:false}]));
  await loadSettings({refresh:true,fetcher:(async()=>new Response(JSON.stringify({settings}),{status:200,headers:{'content-type':'application/json'}})) as Fetcher});
}
const rows=():Email[]=>['a','b','c'].map((id,i)=>mkEmail(id,{subject:`Mail ${id}`,size:1000*(i+1),receivedAt:'2026-10-0'+(i+1)+'T08:00:00Z',threadId:'t1'}));
const source=(over:Partial<ScopeSource>={}):ScopeSource=>({message:[rows()[0]!],conversation:rows(),selection:rows().slice(0,2),folder:{id:'inbox',name:'Inbox',hasSubfolders:true},...over});
function open(props:Partial<Parameters<typeof ExportDialog>[0]>={}) {
  const onClose=vi.fn(),onOpenSettings=vi.fn();
  render(()=><ExportDialog source={source()} onClose={onClose} onOpenSettings={onOpenSettings} {...props}/>);
  return {onClose,onOpenSettings};
}
beforeEach(async()=>{h.calls.length=0;h.status=200;h.error='';h.gate=null;h.saved.length=0;h.body='RAW';
  URL.createObjectURL=vi.fn(()=>'blob:x');URL.revokeObjectURL=vi.fn();await policy({});});
afterEach(()=>{cleanup();clearSettings();});

describe('ExportDialog',()=>{
  it('is a labelled modal dialog and has no axe violations',async()=>{
    const {container}=render(()=><ExportDialog source={source()} onClose={()=>{}}/>) as unknown as {container:HTMLElement};
    const dialog=await screen.findByRole('dialog');
    expect(dialog.getAttribute('aria-modal')).toBe('true');
    expect(dialog.getAttribute('aria-labelledby')).toBeTruthy();
    expect(document.getElementById(dialog.getAttribute('aria-labelledby')!)?.textContent).toBe('Export');
    expect((await axe.run(document.body,{rules:{'color-contrast':{enabled:false},region:{enabled:false}}})).violations.map(v=>v.id)).toEqual([]);
    void container;
  });
  it('closes on Escape and on Close',async()=>{
    const {onClose}=open();
    fireEvent.keyDown(await screen.findByRole('dialog'),{key:'Escape'});
    await waitFor(()=>expect(onClose).toHaveBeenCalled());
  });
  it('offers the scopes the context has and switches between them',async()=>{
    open();
    expect(await screen.findByTestId('export-scope-message')).toBeTruthy();
    expect(screen.getByTestId('export-scope-conversation')).toBeTruthy();
    expect(screen.getByTestId('export-scope-selection')).toBeTruthy();
    expect(screen.getByTestId('export-scope-folder')).toBeTruthy();
    expect(screen.getByTestId('export-estimate').textContent).toMatch(/^1 messages/);
    fireEvent.click(screen.getByTestId('export-scope-conversation'));
    await waitFor(()=>expect(screen.getByTestId('export-estimate').textContent).toMatch(/^3 messages, about 5.9/));
  });
  it('shows no switcher for a lone message',async()=>{
    open({source:{message:[rows()[0]!]}});
    expect(await screen.findByTestId('export-scope-only')).toBeTruthy();
    expect(screen.queryByTestId('export-scope-folder')).toBeNull();
  });
  it('starts on the configured scope when the context offers it',async()=>{
    await policy({'export.defaultScope':'conversation'});
    open();
    expect((await screen.findByTestId('export-scope-conversation') as HTMLInputElement).checked).toBe(true);
  });
  it('lists only the formats the administrator allows, and starts on the default',async()=>{
    await policy({'export.formats':['mbox','csv','pdf'],'export.defaultFormat':'csv'});
    open();
    const select=await screen.findByTestId('export-format') as HTMLSelectElement;
    expect([...select.options].map(option=>option.value)).toEqual(['mbox','csv','pdf']);
    expect(select.value).toBe('csv');
    fireEvent.click(screen.getByTestId('export-scope-selection'));
    await waitFor(()=>expect([...(screen.getByTestId('export-format') as HTMLSelectElement).options].map(option=>option.value)).toEqual(['mbox','csv']));
  });
  it('PDF is a hint and cannot be started',async()=>{
    open();
    fireEvent.change(await screen.findByTestId('export-format'),{target:{value:'pdf'}});
    expect(await screen.findByTestId('export-pdf-hint')).toBeTruthy();
    expect((screen.getByTestId('export-start') as HTMLButtonElement).disabled).toBe(true);
  });
  it('shows a file name example from the pattern and sanitises it',async()=>{
    open({source:{message:[mkEmail('a',{subject:'A/B: c',receivedAt:'2026-10-01T08:00:00Z',from:[{name:'Al',email:'al@example.org'}]})]}});
    fireEvent.click(await screen.findByTestId('export-packaging-zip'));
    const pattern=await screen.findByTestId('export-pattern') as HTMLInputElement;
    fireEvent.input(pattern,{target:{value:'{from}_{subject}'}});
    await waitFor(()=>expect(screen.getByTestId('export-example').textContent).toContain('Al_A_B_ c.eml'));
  });
  it('refuses inline when the selection is over the administrator cap',async()=>{
    await policy({'export.folder.maxMessages':2});
    open();
    fireEvent.click(await screen.findByTestId('export-scope-conversation'));
    const refusal=await screen.findByTestId('export-refusal');
    expect(refusal.textContent).toContain('Too many messages');
    expect(screen.getByRole('alert')).toBe(refusal);
    expect((screen.getByTestId('export-start') as HTMLButtonElement).disabled).toBe(true);
  });
  it('refuses when downloads are switched off',async()=>{
    await policy({'export.message.download.enabled':false});
    open();
    expect((await screen.findByTestId('export-refusal')).textContent).toContain('switched off');
    expect((screen.getByTestId('export-start') as HTMLButtonElement).disabled).toBe(true);
  });
  it('asks the server for a folder estimate and shows its verdict',async()=>{
    h.error='exportTooLarge';
    open({scope:'folder'});
    await waitFor(()=>expect(h.calls.some(url=>url.includes('/api/export/folder/inbox.zip')&&url.includes('estimate=1'))).toBe(true));
    expect((await screen.findByTestId('export-refusal')).textContent).toContain('larger than your administrator allows');
    expect((screen.getByTestId('export-start') as HTMLButtonElement).disabled).toBe(true);
  });
  it('exports one message through the export route and offers Download again',async()=>{
    open({source:{message:[rows()[0]!]}});
    fireEvent.click(await screen.findByTestId('export-start'));
    await screen.findByTestId('export-result');
    expect(h.calls[0]).toMatch(/\/api\/export\/a\?format=eml$/);
    expect(h.saved).toEqual(['2026-10-01-a@example.org-Mail a.eml']);
    fireEvent.click(screen.getByTestId('export-again'));
    expect(h.saved).toHaveLength(2);
  });
  it('packs several messages into a ZIP with a file per message',async()=>{
    open({scope:'selection'});
    fireEvent.click(await screen.findByTestId('export-start'));
    await screen.findByTestId('export-result');
    expect(h.calls.filter(url=>url.includes('/api/export/')&&!url.includes('folder'))).toHaveLength(2);
    expect(h.saved[0]).toMatch(/\.zip$/);
  });
  it('shows progress and cancels without saving',async()=>{
    let release=()=>{};h.gate=new Promise<void>(resolve=>{release=resolve;});
    open({scope:'selection'});
    fireEvent.click(await screen.findByTestId('export-start'));
    const bar=await screen.findByTestId('export-progress');
    expect(bar.querySelector('progress')).toBeTruthy();
    fireEvent.click(screen.getByTestId('export-cancel'));
    release();
    await waitFor(()=>expect(screen.queryByTestId('export-progress')).toBeNull());
    expect(h.saved).toEqual([]);
    expect(screen.queryByTestId('export-error')).toBeNull();
  });
  it('cancels a multi-message export when the mailbox changes during fetch',async()=>{
    let release=()=>{};h.gate=new Promise<void>(resolve=>{release=resolve;});
    const {onClose}=open({scope:'selection'});
    fireEvent.click(await screen.findByTestId('export-start'));
    await waitFor(()=>expect(h.calls).toHaveLength(1));
    installLinkedMailbox(null);release();
    await waitFor(()=>expect(onClose).toHaveBeenCalled());
    await waitFor(()=>expect(screen.queryByTestId('export-progress')).toBeNull());
    expect(h.calls).toHaveLength(1);expect(h.saved).toEqual([]);
  });
  it('shows the server reason with a Retry that runs again',async()=>{
    h.status=403;h.error='legalHold';
    open({source:{message:[rows()[0]!]}});
    fireEvent.click(await screen.findByTestId('export-start'));
    expect((await screen.findByTestId('export-error')).textContent).toContain('legal hold');
    h.status=200;
    fireEvent.click(screen.getByTestId('export-retry'));
    await screen.findByTestId('export-result');
  });
  it('builds CSV in the page without calling the server',async()=>{
    open({scope:'selection'});
    fireEvent.change(await screen.findByTestId('export-format'),{target:{value:'csv'}});
    fireEvent.click(screen.getByTestId('export-packaging-single'));
    fireEvent.click(screen.getByTestId('export-start'));
    await screen.findByTestId('export-result');
    expect(h.calls).toEqual([]);
    expect(h.saved[0]).toMatch(/\.csv$/);
  });
  it('links to the Settings page without navigating there on open',async()=>{
    const {onClose,onOpenSettings}=open();
    expect(onOpenSettings).not.toHaveBeenCalled();
    fireEvent.click(await screen.findByTestId('export-more'));
    expect(onClose).toHaveBeenCalled();expect(onOpenSettings).toHaveBeenCalledTimes(1);
  });
});

describe('problemKey',()=>{
  it('maps codes, then statuses, then falls back',()=>{
    expect(problemKey(new ExportError('legalHold',403))).toBe('export-problem-legalHold');
    expect(problemKey(new ExportError('http-501',501))).toBe('export-problem-exportRequiresEngine');
    expect(problemKey(new ExportError('http-429',429))).toBe('export-problem-rateLimited');
    expect(problemKey(new ExportError('odd',403))).toBe('export-problem-forbidden');
    expect(problemKey(new ExportError('odd',404))).toBe('export-problem-gone');
    expect(problemKey(new Error('x'))).toBe('export-problem-failed');
  });
});
