import {describe,expect,it} from 'vitest';
import {allowedOverrides,exportFormats,folderSections,maskHost,matchesShortcut,messageSections,notesMaxLength,permanentDeletePolicy,remoteForDisplay,remoteVisibility,sizeCompute} from './policy.ts';
import {effectiveOverride,OVERRIDE_DEFS,withOverride} from './folderOverrides.ts';
import {filterHeaders,headerList,headersOnly,hopBars,hopDelayText,spamVerdict} from './messageProps.ts';
import {highestPriority,priorityOf,showsPriorityMark} from '../messagePriority.ts';
import {clearSettings,loadSettings,type Fetcher} from '../../api/settings.ts';

const reader=(values:Record<string,unknown>)=>(key:string)=>values[key];

describe('policy readers',()=>{
 it('default to everything on when nothing is set',()=>{
  expect(folderSections(reader({}))).toHaveLength(7);
  expect(messageSections(reader({}))).toHaveLength(5);
  expect(allowedOverrides(reader({}))).toHaveLength(11);
  expect(allowedOverrides(reader({}))).toContain('hideWhenEmpty');
  expect(permanentDeletePolicy(reader({}))).toBe('allow');
  expect(remoteVisibility(reader({}))).toBe('masked-host');
  expect(sizeCompute(reader({}))).toBe('on-open');
  expect(exportFormats(reader({}))).toEqual(['eml','mbox']);
  expect(notesMaxLength(reader({}))).toBe(2000);
 });
 it('follow the listed sections and the dependent switches',()=>{
  expect(folderSections(reader({'properties.folder.sections':['general','notes','overrides'],'properties.folder.notes.enabled':false}))).toEqual(['general','overrides']);
  expect(folderSections(reader({'properties.folder.overrides.allowed':[]}))).not.toContain('overrides');
  expect(messageSections(reader({'properties.headers.raw':false,'properties.scoring.show':false}))).toEqual(['summary','attachments','actions']);
  expect(exportFormats(reader({'export.folder.format':'zip-mbox'}))).toEqual(['mbox']);
  expect(notesMaxLength(reader({'properties.folder.notes.maxLength':500}))).toBe(500);
  expect(notesMaxLength(reader({'properties.folder.notes.maxLength':99999}))).toBe(2000);
 });
 it('masks and reduces the remote location',()=>{
  const remote={path:'INBOX',host:'imap.mail.example.org',uidValidity:3};
  expect(remoteForDisplay(remote,'full')).toEqual(remote);
  expect(remoteForDisplay(remote,'masked-host')).toEqual({path:'INBOX',host:'i***.example.org',uidValidity:3});
  expect(remoteForDisplay(remote,'path-only')).toEqual({path:'INBOX',uidValidity:3});
  expect(remoteForDisplay(remote,'hidden')).toBeNull();
  expect(maskHost('localhost')).toBe('l***');
 });
 it('matches the Properties shortcut exactly',()=>{
  const key=(over:Partial<KeyboardEvent>)=>({key:'Enter',altKey:true,ctrlKey:false,metaKey:false,shiftKey:false,...over});
  expect(matchesShortcut(key({}),'Alt+Enter')).toBe(true);
  expect(matchesShortcut(key({altKey:false}),'Alt+Enter')).toBe(false);
  expect(matchesShortcut(key({shiftKey:true}),'Alt+Enter')).toBe(false);
  expect(matchesShortcut(key({key:'p',altKey:false,ctrlKey:true}),'Ctrl+P')).toBe(true);
 });
});

describe('overrides',()=>{
 const def=OVERRIDE_DEFS.find(item=>item.key==='counters')!;
 const entry=(value:unknown,over:Record<string,unknown>={})=>({value,source:'user' as const,locked:false,...over});
 it('inherits the user setting, then the folder override wins',()=>{
  const read=()=>entry('both');
  expect(effectiveOverride(def,undefined,read,['counters'])).toMatchObject({value:'both',source:'user',overridden:false});
  expect(effectiveOverride(def,{counters:'total'},read,['counters'])).toMatchObject({value:'total',source:'folder',overridden:true});
 });
 it('reports an administrator default, and a lock beats the folder override',()=>{
  expect(effectiveOverride(def,undefined,()=>entry('unread',{source:'global'}),['counters'])).toMatchObject({source:'admin-default',locked:false});
  expect(effectiveOverride(def,{counters:'total'},()=>entry('unread',{source:'global',locked:true}),['counters'])).toMatchObject({value:'unread',source:'admin-locked',locked:true,overridden:false});
 });
 it('ignores a stored override the administrator no longer allows',()=>{
  expect(effectiveOverride(def,{counters:'total'},()=>entry('both'),[])).toMatchObject({value:'both',source:'user'});
 });
 it('inherits the shipped empty-folder visibility default and respects explicit false, reset and admin lock',()=>{
  const visibility=OVERRIDE_DEFS.find(item=>item.key==='hideWhenEmpty')!;
  expect(effectiveOverride(visibility,undefined,()=>undefined,['hideWhenEmpty'])).toMatchObject({value:false,source:'user',overridden:false});
  const read=()=>entry(true,{source:'global'});
  expect(effectiveOverride(visibility,undefined,read,['hideWhenEmpty'])).toMatchObject({value:true,source:'admin-default',overridden:false});
  const own=withOverride(undefined,'f1',{hideWhenEmpty:false});
  expect(own).toEqual({f1:{hideWhenEmpty:false}});
  expect(effectiveOverride(visibility,own['f1'],read,['hideWhenEmpty'])).toMatchObject({value:false,source:'folder',overridden:true});
  expect(effectiveOverride(visibility,own['f1'],()=>entry(true,{source:'global',locked:true}),['hideWhenEmpty'])).toMatchObject({value:true,source:'admin-locked',overridden:false,locked:true});
  expect(effectiveOverride(visibility,own['f1'],read,[])).toMatchObject({value:true,source:'admin-default',overridden:false});
  expect(withOverride(own,'f1',{hideWhenEmpty:undefined})).toEqual({});
 });
 it('writes and clears overrides, dropping an empty folder entry',()=>{
  const one=withOverride(undefined,'f1',{counters:'none',notes:'x'});
  expect(one).toEqual({f1:{counters:'none',notes:'x'}});
  expect(withOverride(one,'f1',{counters:undefined})).toEqual({f1:{notes:'x'}});
  expect(withOverride({f1:{counters:'none'}},'f1',{counters:undefined})).toEqual({});
 });
});

describe('message header parsing',()=>{
 const raw='Received: from a by b;\r\n Tue, 1 Jul 2026\r\nX-Spam-Status: Yes, score=7.1\r\nX-Spam-Flag: YES\r\nSubject: hi\r\n\r\nbody: not a header';
 it('lists unfolded headers in order and stops at the body',()=>{
  const list=headerList(raw);
  expect(list.map(h=>h.name)).toEqual(['Received','X-Spam-Status','X-Spam-Flag','Subject']);
  expect(list[0]!.value).toBe('from a by b; Tue, 1 Jul 2026');
 });
 it('filters by name or value',()=>{
  const list=headerList(raw);
  expect(filterHeaders(list,'spam').map(h=>h.name)).toEqual(['X-Spam-Status','X-Spam-Flag']);
  expect(filterHeaders(list,'hi').map(h=>h.name)).toContain('Subject');
  expect(filterHeaders(list,'')).toHaveLength(4);
 });
 it('reads the spam status and flag',()=>{
  const v=spamVerdict(raw);
  expect(v.status).toBe('Yes, score=7.1');
  expect(v.flag).toBe(true);
 });
 it('writes the header block alone with CRLF',()=>{
  expect(headersOnly('A: 1\nB: 2\n\nbody')).toBe('A: 1\r\nB: 2\r\n\r\n');
 });
 it('formats hop delays and scales the bars',()=>{
  expect(hopDelayText(null)).toBeNull();
  expect(hopDelayText(400)).toBe('400 ms');
  expect(hopDelayText(5000)).toBe('5.0 s');
  expect(hopDelayText(125_000)).toBe('2 min');
  expect(hopDelayText(3_900_000)).toBe('1 h 5 min');
  expect(hopBars([null,1000,4000])).toEqual([0,25,100]);
  expect(hopBars([0,0])).toEqual([0,0]);
 });
});

describe('sender-set priority trust',()=>{
 const json=(body:unknown)=>new Response(JSON.stringify(body),{status:200,headers:{'content-type':'application/json'}});
 const withTrust=async(value:string)=>{clearSettings();await loadSettings({refresh:true,fetcher:(async()=>json({settings:{'mail.priority.trustSenderHeaders':{value,source:'global',locked:true}}})) as Fetcher});};
 const aligned={dkim:{result:'pass',domain:'x.org'},dmarc:{result:'pass',aligned:true}};
 it('"always" honours it, "never" ignores it, "trusted" needs an aligned DMARC pass',async()=>{
  await withTrust('always');
  expect(priorityOf({mailwomanPriority:'high'})).toBe('high');
  await withTrust('never');
  expect(priorityOf({mailwomanPriority:'high',mailwomanAuth:aligned})).toBe('normal');
  await withTrust('trusted');
  expect(priorityOf({mailwomanPriority:'high'})).toBe('normal');
  expect(priorityOf({mailwomanPriority:'high',mailwomanAuth:{...aligned,dmarc:{result:'pass',aligned:false}}})).toBe('normal');
  expect(priorityOf({mailwomanPriority:'high',mailwomanAuth:aligned})).toBe('high');
  expect(highestPriority([{mailwomanPriority:'low',mailwomanAuth:aligned},{mailwomanPriority:'high'}])).toBe('low');
  expect(showsPriorityMark('low','off')).toBe(false);
 });
});
