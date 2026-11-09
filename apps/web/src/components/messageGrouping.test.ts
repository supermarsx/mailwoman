import {describe,it,expect} from 'vitest';
import {mkEmail} from './appHarness.tsx';
import {groupMessageRows,groupedWindow,groupedIndexAt,groupedRowTop} from './messageGrouping.ts';
import {setActiveLocale} from '../i18n/registry.ts';

describe('message category grouping',()=>{
 it('formats date headings in the selected app locale while keeping stable timezone bucket keys',()=>{
  const emails=[mkEmail('locale',{receivedAt:'2026-11-09T00:30:00Z'})];
  setActiveLocale('en');const english=groupMessageRows(emails,new Set(),'received',72,'UTC').headers[0]!;
  try{
   setActiveLocale('pt-PT');const portuguese=groupMessageRows(emails,new Set(),'received',72,'UTC').headers[0]!;
   expect(english.label).toContain('November');expect(portuguese.label).toContain('novembro');expect(portuguese.key).toBe(english.key);
   const previousDay=groupMessageRows(emails,new Set(),'received',72,'America/New_York').headers[0]!;
   expect(previousDay.key).not.toBe(portuguese.key);expect(previousDay.label).toContain('novembro');
  }finally{setActiveLocale('en');}
 });
 it('groups whole conversations, retaining their selection identity and children',()=>{
  const a=mkEmail('a',{threadId:'thread',receivedAt:'2026-01-02T00:00:00Z',from:[{name:'Alice',email:'alice@example.org'}]});
  const b=mkEmail('b',{threadId:'thread',receivedAt:'2026-01-01T00:00:00Z',from:[{name:'Bob',email:'bob@example.org'}]});
  const c=mkEmail('c',{from:[{name:'Bob',email:'bob@example.org'}]});
  const layout=groupMessageRows([a,b,c],new Set(['thread']),'sender',72);
  expect(layout.headers.map(header=>[header.label,header.count])).toEqual([['Alice <alice@example.org>',2],['Bob <bob@example.org>',1]]);
  expect(layout.rows.map(row=>row.email.id)).toEqual(['a','b','a','c']);
  expect(layout.rows[0]?.key).toBe('thread');
  expect(layout.tops).toEqual([32,104,176,280]);
  expect(layout.height).toBe(352);
 });
 it('windows messages around short headings and keeps the unfetched tail aligned',()=>{
  const emails=Array.from({length:50},(_,i)=>mkEmail(String(i),{from:[{name:null,email:`sender-${i}@example.org`}]}));
  const layout=groupMessageRows(emails,new Set(),'sender',72);
  const win=groupedWindow(layout,1040,600,72,100);
  expect(groupedIndexAt(layout,1040,72)).toBe(10);
  expect(win.startIndex).toBe(4);
  expect(win.endIndex).toBeGreaterThan(15);
  expect(win.totalHeight).toBe(50*104+50*72);
  expect(groupedRowTop(layout,50,72)).toBe(5200);
  expect(groupedRowTop(layout,51,72)).toBe(5272);
  expect(groupedIndexAt(layout,5272,72)).toBe(51);
 });
 it('has identical geometry without grouping, and groups normalized subjects and sizes',()=>{
  const emails=[mkEmail('a',{subject:'Re: Plans',size:20}),mkEmail('b',{subject:'FW: Plans',size:30}),mkEmail('c',{subject:'Other',size:11*1024*1024})];
  const normal=groupMessageRows(emails,new Set(),'none',72);
  expect(normal.headers).toEqual([]);expect(normal.tops).toEqual([0,72,144]);expect(normal.height).toBe(216);
  expect(groupMessageRows(emails,new Set(),'subject',72).headers.map(header=>header.count)).toEqual([2,1]);
  expect(groupMessageRows(emails,new Set(),'size',72).headers.map(header=>header.key)).toEqual(['tiny','huge']);
 });
});
