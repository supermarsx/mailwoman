import {describe,it,expect} from 'vitest';
import {swapQuote} from './quoteRegion.ts';

const plain='<p></p><p>Alice wrote:</p><blockquote><p>original text</p></blockquote>';
const rich='<p></p><p>Alice wrote:</p><blockquote><p>original text</p><img src="cid:r1@mailwoman"></blockquote>';
const count=(html:string,needle:string)=>html.split(needle).length-1;

describe('swapQuote',()=>{
 it('replaces the quote of an untouched reply',()=>{
  const out=swapQuote(plain,rich,'Alice wrote:','reply')!;
  expect(count(out,'original text')).toBe(1);expect(count(out,'Alice wrote:')).toBe(1);expect(out).toContain('cid:r1@mailwoman');
 });
 it('keeps text typed above and below the quote and a signature',()=>{
  const current='<p>My answer</p><p>Alice wrote:</p><blockquote><p>original text</p></blockquote><p>-- <br>Sig</p>';
  const out=swapQuote(current,rich,'Alice wrote:','reply')!;
  expect(out.startsWith('<p>My answer</p>')).toBe(true);expect(out).toContain('Sig');
  expect(count(out,'original text')).toBe(1);expect(count(out,'<blockquote')).toBe(1);expect(out).toContain('<img');
 });
 it('is idempotent: swapping twice leaves one copy',()=>{
  const once=swapQuote(plain,rich,'Alice wrote:','reply')!;
  const twice=swapQuote(once,rich,'Alice wrote:','reply')!;
  expect(count(twice,'original text')).toBe(1);expect(count(twice,'<img')).toBe(1);
 });
 it('swaps a forward from its header to the end',()=>{
  const fwd=(body:string)=>'<p></p><p>---- Forwarded ----<br>From: a</p>'+body;
  const out=swapQuote('<p>FYI</p>'+fwd('<p>original text</p>'),fwd('<p>original text</p><img src="cid:x">'),'---- Forwarded ----','tail')!;
  expect(count(out,'original text')).toBe(1);expect(count(out,'Forwarded')).toBe(1);expect(out).toContain('cid:x');expect(out.startsWith('<p>FYI</p>')).toBe(true);
 });
 it('returns null when the user deleted the quote',()=>{
  expect(swapQuote('<p>just my text</p>',rich,'Alice wrote:','reply')).toBeNull();
 });
});
