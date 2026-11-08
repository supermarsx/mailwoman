import {describe,expect,it} from 'vitest';
import type {Email} from '../../api/jmap-types.ts';
import {buildCsv,buildJson,estimateFor,exportPolicy,fileNames,fileStem,offeredScopes,packagings,sanitizeName,scopeFormats,startScope,stripHeaders} from './exportPlan.ts';

const mail=(id:string,over:Partial<Email>={}):Email=>({id,mailboxIds:{inbox:true},from:[{name:'Alice',email:'alice@example.org'}],to:[{name:null,email:'me@example.org'}],subject:'Hello',receivedAt:'2026-10-05T08:00:00Z',size:100,keywords:{},...over} as Email);
const read=(values:Record<string,unknown>)=>(key:string)=>values[key];
const text=(bytes:Uint8Array)=>new TextDecoder('latin1').decode(bytes);

describe('policy',()=>{
  it('reads the shipped defaults when nothing is loaded',()=>{
    const policy=exportPolicy(read({}));
    expect(policy.enabled).toBe(true);expect(policy.formats).toHaveLength(9);expect(policy.defaultFormat).toBe('eml');expect(policy.defaultScope).toBe('message');
    expect(policy.maxMessages).toBe(100000);
  });
  it('limits the formats and falls back when the default is not offered',()=>{
    const policy=exportPolicy(read({'export.formats':['mbox','csv','nonsense'],'export.defaultFormat':'eml','export.defaultScope':'folder'}));
    expect(policy.formats).toEqual(['mbox','csv']);expect(policy.defaultFormat).toBe('mbox');expect(policy.defaultScope).toBe('folder');
  });
  it('honours the switches and the caps',()=>{
    const policy=exportPolicy(read({'export.dialog.enabled':false,'export.message.download.enabled':false,'export.folder.maxMessages':5,'export.folder.maxBytes':1000}));
    expect(policy).toMatchObject({enabled:false,downloadEnabled:false,maxMessages:5,maxBytes:1000});
    expect(scopeFormats('message',policy)).toEqual([]);
  });
});

describe('scopes and formats',()=>{
  const policy={folderEnabled:true};
  it('offers only the scopes the context has',()=>{
    expect(offeredScopes({message:[mail('a')]},policy)).toEqual(['message']);
    expect(offeredScopes({message:[mail('a')],conversation:[mail('a'),mail('b')],selection:[mail('a'),mail('b'),mail('c')],folder:{id:'inbox',name:'Inbox'}},policy)).toEqual(['message','conversation','selection','folder']);
    expect(offeredScopes({message:[mail('a')],conversation:[mail('a')],folder:{id:'inbox',name:'Inbox'}},{folderEnabled:false})).toEqual(['message']);
  });
  it('starts on the wanted scope when offered',()=>{
    expect(startScope(['message','folder'],'folder')).toBe('folder');
    expect(startScope(['message'],'folder')).toBe('message');
  });
  it('keeps PDF to one message and a folder to eml or mbox',()=>{
    const all={formats:['eml','mbox','txt','csv','pdf'] as const,downloadEnabled:true};
    expect(scopeFormats('message',all)).toEqual(['eml','mbox','txt','csv','pdf']);
    expect(scopeFormats('selection',all)).toEqual(['eml','mbox','txt','csv']);
    expect(scopeFormats('folder',all)).toEqual(['eml','mbox']);
  });
  it('allows one file for many messages only in mbox, csv and json',()=>{
    expect(packagings('selection','eml',5)).toEqual(['zip']);
    expect(packagings('selection','mbox',5)).toEqual(['single','zip']);
    expect(packagings('message','docx',1)).toEqual(['single','zip']);
    expect(packagings('folder','eml',0)).toEqual(['zip']);
    expect(packagings('message','pdf',1)).toEqual([]);
  });
});

describe('file names',()=>{
  it('replaces characters a file system rejects',()=>{
    expect(sanitizeName('a/b\\c:d*e?f"g<h>i|j')).toBe('a_b_c_d_e_f_g_h_i_j');
    expect(sanitizeName('tab\there\u0000x')).toBe('tab_here_x');
  });
  it('strips edge dots and spaces and guards device names and empties',()=>{
    expect(sanitizeName('  ..hidden.. ')).toBe('hidden');
    expect(sanitizeName('CON')).toBe('_CON');
    expect(sanitizeName('...')).toBe('message');
    expect(sanitizeName('../../etc/passwd')).toBe('_.._etc_passwd');
  });
  it('cuts to the longest name by characters and clamps absurd limits',()=>{
    expect(Array.from(sanitizeName('x'.repeat(300),40))).toHaveLength(40);
    expect(Array.from(sanitizeName('x'.repeat(300),5))).toHaveLength(20);
    expect(Array.from(sanitizeName('x'.repeat(500),9999))).toHaveLength(200);
    expect(Array.from(sanitizeName('\u{1F600}'.repeat(50),30))).toHaveLength(30);
  });
  it('expands the tokens',()=>{
    const email=mail('m1',{subject:'Re: Plan / budget'});
    expect(fileStem('{date}-{from}-{subject}',email,1)).toBe('2026-10-05-Alice-Re_ Plan _ budget');
    expect(fileStem('{n}_{id}',email,7)).toBe('7_m1');
    expect(fileStem('',email,1)).toBe('2026-10-05-Alice-Re_ Plan _ budget');
  });
  it('handles messages without a subject, sender or date',()=>{
    expect(fileStem('{date}-{from}-{subject}',mail('x',{subject:'',from:[],receivedAt:''}),1)).toBe('undated-unknown-no subject');
  });
  it('numbers names that collide, ignoring case',()=>{
    expect(fileNames([mail('a'),mail('b',{subject:'HELLO'}),mail('c')],'{subject}','eml')).toEqual(['Hello.eml','HELLO (2).eml','Hello (3).eml']);
  });
});

describe('estimate',()=>{
  const caps={maxBytes:1000,maxMessages:3};
  it('sums the sizes and passes under the caps',()=>{
    expect(estimateFor([{size:100},{size:200}],caps)).toEqual({messages:2,bytes:300,allowed:true});
  });
  it('refuses over a cap with the reason',()=>{
    expect(estimateFor([{size:1},{size:1},{size:1},{size:1}],caps)).toMatchObject({allowed:false,reason:'tooManyMessages'});
    expect(estimateFor([{size:2000}],caps)).toMatchObject({allowed:false,reason:'tooLarge'});
    expect(estimateFor([],caps)).toMatchObject({allowed:false,reason:'nothing'});
  });
  it('treats a missing size as zero',()=>{expect(estimateFor([{}],caps).bytes).toBe(0);});
});

describe('header privacy',()=>{
  const raw='From a@b Mon Oct  5 08:00:00 2026\r\nReceived: from x\r\n by y;\r\n Mon, 5 Oct\r\nFrom: a@b\r\nBcc: secret@example.org\r\nSubject: Hi\r\n\r\nReceived: this is body text\r\nBcc: also body\r\n';
  const bytes=new TextEncoder().encode(raw);
  it('removes folded Received and Bcc fields and keeps the body untouched',()=>{
    const out=text(stripHeaders(bytes,{stripReceived:true,redactBcc:true}));
    expect(out).toBe('From a@b Mon Oct  5 08:00:00 2026\r\nFrom: a@b\r\nSubject: Hi\r\n\r\nReceived: this is body text\r\nBcc: also body\r\n');
  });
  it('removes only what was asked for',()=>{
    expect(text(stripHeaders(bytes,{stripReceived:false,redactBcc:true}))).toContain('Received: from x');
    expect(text(stripHeaders(bytes,{stripReceived:false,redactBcc:true}))).not.toContain('secret@example.org');
    expect(stripHeaders(bytes,{stripReceived:false,redactBcc:false})).toBe(bytes);
  });
  it('keeps 8-bit bytes intact',()=>{
    const data=Uint8Array.from([...new TextEncoder().encode('Bcc: x\nSubject: y\n\n'),0xff,0x00,0xe9]);
    const out=stripHeaders(data,{stripReceived:false,redactBcc:true});
    expect([...out.slice(-3)]).toEqual([0xff,0x00,0xe9]);
  });
});

describe('csv and json',()=>{
  it('writes the summary columns, quoting commas, quotes and newlines',()=>{
    const csv=buildCsv([mail('a',{subject:'Hi, "you"\nnow',size:12})]);
    expect(csv.startsWith('﻿date,from,to,subject,size\r\n')).toBe(true);
    expect(csv).toContain('"Hi, ""you""\nnow"');
    expect(csv).toContain('Alice <alice@example.org>');
  });
  it('defuses spreadsheet formulas',()=>{
    expect(buildCsv([mail('a',{subject:'=HYPERLINK("x")'})])).toContain('"\'=HYPERLINK(""x"")"');
  });
  it('writes JSON properties and can leave Bcc out',()=>{
    const withBcc=[mail('a',{bcc:[{name:null,email:'b@example.org'}]})];
    const rows=JSON.parse(buildJson(withBcc)) as Record<string,unknown>[];
    expect(rows[0]).toMatchObject({id:'a',subject:'Hello'});expect(rows[0]!['bcc']).toBeTruthy();
    expect(Object.keys((JSON.parse(buildJson(withBcc,{redactBcc:true})) as Record<string,unknown>[])[0]!)).not.toContain('bcc');
  });
});
