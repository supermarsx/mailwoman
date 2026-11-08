import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import type {Email} from '../../api/jmap-types.ts';
import {clearSettings,loadSettings,type Fetcher} from '../../api/settings.ts';
import {mkEmail} from '../appHarness.tsx';
import {settingsRequest} from '../../state/settingsRequest.ts';
import {closeExportDialog,exportDialogOpen} from './host.tsx';
import {exportFromMenu,sourceFor,type ExportApp} from './openExport.ts';

async function policy(values:Record<string,unknown>):Promise<void> {
  const settings=Object.fromEntries(Object.entries(values).map(([key,value])=>[key,{value,source:'global',locked:false}]));
  await loadSettings({refresh:true,fetcher:(async()=>new Response(JSON.stringify({settings}),{status:200,headers:{'content-type':'application/json'}})) as Fetcher});
}
const rows:Email[]=[mkEmail('a',{threadId:'t1'}),mkEmail('b',{threadId:'t1'}),mkEmail('c',{threadId:'t2'})];
const app=(selected:string[]=[]):ExportApp=>({
  listMessages:()=>rows,selectedIds:()=>new Set(selected),selectedMailboxId:()=>'inbox',
  mailboxes:()=>[{id:'inbox',name:'Inbox',parentId:null},{id:'sub',name:'Sub',parentId:'inbox'}],
  exportMessage:vi.fn(async()=>undefined),
});
beforeEach(async()=>{await policy({});});
afterEach(()=>{closeExportDialog();clearSettings();});

describe('sourceFor',()=>{
  it('offers the thread, the selection that holds the message, and the open folder',()=>{
    const source=sourceFor(app(['a','c']),rows[0]!);
    expect(source.message.map(row=>row.id)).toEqual(['a']);
    expect(source.conversation?.map(row=>row.id)).toEqual(['a','b']);
    expect(source.selection?.map(row=>row.id)).toEqual(['a','c']);
    expect(source.folder).toEqual({id:'inbox',name:'Inbox',hasSubfolders:true});
  });
  it('leaves out a one-message thread and a selection that does not hold the message',()=>{
    const source=sourceFor(app(['b','c']),rows[2]!);
    expect(source.conversation).toBeUndefined();
    expect(source.selection?.map(row=>row.id)).toEqual(['b','c']);
    expect(sourceFor(app(['a']),rows[2]!).selection).toBeUndefined();
  });
});

describe('menu wiring',()=>{
  it('opens the dialog in place and does not ask for the Settings screen',()=>{
    const before=settingsRequest();
    expect(exportFromMenu(app(),'a')).toBe(true);
    expect(exportDialogOpen()).toBe(true);
    expect(settingsRequest()).toBe(before);
  });
  it('returns false, so the caller downloads the .eml, when the dialog is switched off or the message is unknown',async()=>{
    await policy({'export.dialog.enabled':false});
    expect(exportFromMenu(app(),'a')).toBe(false);
    expect(exportDialogOpen()).toBe(false);
    await policy({});
    expect(exportFromMenu(app(),'nope')).toBe(false);
  });
  it('keeps the reader and list menus off the Settings export screen',()=>{
    const mailbox=readFileSync(resolve(process.cwd(),'src/screens/Mailbox.tsx'),'utf8');
    const start=mailbox.indexOf('async function openExport'),body=mailbox.slice(start,mailbox.indexOf('/** A message opened from the Mail list',start));
    expect(body).toContain('exportFromMenu');
    expect(body).not.toContain('mail-transfer');
    expect(body).not.toContain('openSettings');
    const menu=readFileSync(resolve(process.cwd(),'src/components/MessageContextMenu.tsx'),'utf8');
    expect(menu).toContain('exportFromMenu(app,id,message)');
  });
});
