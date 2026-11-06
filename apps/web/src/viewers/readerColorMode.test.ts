import {beforeEach,describe,expect,it} from 'vitest';
import {messageColorOverride,readerColorKey,resetMessageColorOverrides,resolveReaderColorMode,setMessageColorOverride,READER_COLOR_CHOICES} from './readerColorMode.ts';
import {bodyFrameDoc} from './sandbox.ts';
import {diffReaderDocument,patchPrefixes} from './readerPatch.ts';

const doc=(colorMode:'original'|'adaptive'|'invert',html:string)=>bodyFrameDoc('full-sanitized',{html},{appearance:{colorMode,fontSize:14,lineHeight:1.6,contentWidth:'full'},themeVars:colorMode==='adaptive'?':root{--mw-color-bg:#111}':''});

describe('per-message color override store',()=>{
 beforeEach(()=>resetMessageColorOverrides());
 it('is keyed by account and message, clears with null, and persists for the session',()=>{
  const a=readerColorKey('acc','m1'),b=readerColorKey('acc','m2');
  setMessageColorOverride(a,'invert');
  expect(messageColorOverride(a)).toBe('invert');expect(messageColorOverride(b)).toBeUndefined();
  expect(JSON.parse(sessionStorage.getItem('mw.reader.colorOverrides')!)).toEqual({[a]:'invert'});
  setMessageColorOverride(a,null);expect(messageColorOverride(a)).toBeUndefined();
 });
 it('auto follows the theme; the others are themselves',()=>{
  expect(resolveReaderColorMode('auto','dark')).toBe('adaptive');expect(resolveReaderColorMode('auto','light')).toBe('original');
  for(const c of READER_COLOR_CHOICES.filter(c=>c!=='auto'))expect(resolveReaderColorMode(c,'dark')).toBe(c);
 });
});

describe('colour mode changes hot-patch the frame (no reload)',()=>{
 const body='<p style="margin:0">Hello <img src="data:image/png;base64,AAAA"></p>';
 const prefixes=patchPrefixes('https://x');
 it('original to invert and back differ only in the stylesheet text',()=>{
  const a=doc('original',body),b=doc('invert',body);
  const ops=diffReaderDocument(a,b,prefixes);
  expect(ops).not.toBeNull();expect(ops!.length).toBe(1);expect(ops![0]).toMatchObject({s:'h',a:'text'});
  expect(ops![0]!.v).toContain('invert(1)');
  expect(diffReaderDocument(b,a,prefixes)!.map(o=>o.a)).toEqual(['text']);
 });
 it('original to adaptive on a body with no sender paint attributes is a text patch',()=>{
  expect(diffReaderDocument(doc('original',body),doc('adaptive',body),prefixes)!.map(o=>o.a)).toEqual(['text']);
 });
});
