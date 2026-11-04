import {afterEach,beforeAll,describe,expect,it,vi} from 'vitest';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {initSync} from '../../wasm/mw-sanitize/mw_sanitize.js';
import {cleanup,fireEvent,render,screen} from '@solidjs/testing-library';
import {quoteForForward,quoteForOutlook,quoteForReply} from './quote.ts';
import {normalizeHtmlSource} from './htmlSource.ts';
import {NativeHtmlVisualEditor,type NativeHtmlVisualApi} from './NativeHtmlVisualEditor.tsx';
import {docFromHtml,htmlFromDoc} from './richtext.ts';
import {EditorState,TextSelection} from 'prosemirror-state';

afterEach(()=>{cleanup();vi.restoreAllMocks();});
beforeAll(()=>initSync({module:readFileSync(resolve(process.cwd(),'src/wasm/mw-sanitize/mw_sanitize_bg.wasm'))}));

const HOSTILE='<html><head><style>p{color:red;font-size:30px} body{background:#000;color:#fff} *{font-family:Comic Sans MS}</style></head><body dir="rtl" style="text-align:right;font-family:Georgia"><div style="color:#0f0"><font face="Arial" color="red">Original</font><p>second</p></div></body></html>';
const OWNED=new Set(['logo']);

function mount(html:string){
 let api!:NativeHtmlVisualApi;
 render(()=><NativeHtmlVisualEditor html={html} revision={0} reply owned={()=>OWNED} onInput={()=>undefined} onReady={value=>api=value}/>);
 const frame=screen.getByTitle('Visual editor') as HTMLIFrameElement;
 const doc=frame.contentDocument!,view=frame.contentWindow as Window&{CSSStyleSheet:typeof CSSStyleSheet};
 if(!view.CSSStyleSheet.prototype.replaceSync)Object.defineProperty(view.CSSStyleSheet.prototype,'replaceSync',{value:()=>{},configurable:true});
 doc.documentElement.setAttribute('data-mw-native-document','true');fireEvent.load(frame);
 return {api,doc};
}

describe('a reply keeps its own formatting apart from the quoted original',()=>{
 it('scopes the original rules and wrapper to a quote-only container',async()=>{
  const quote=await quoteForReply({html:HOSTILE,ownedCids:OWNED},'A wrote:');
  const doc=new DOMParser().parseFromString(quote.html,'text/html');
  const answer=doc.body.firstElementChild!;
  expect(answer.tagName).toBe('P');expect(answer.textContent).toBe('');
  expect(doc.querySelector('.mw-email-body')).toBeNull();
  const wrapper=doc.querySelector('blockquote .mw-quote-body')!;
  expect(wrapper.getAttribute('dir')).toBe('rtl');
  const css=doc.querySelector('style')!.textContent!;
  expect(css).toContain('.mw-quote-body p{color:red;font-size:30px}');
  expect(css).toContain('.mw-quote-body{background:#000;color:#fff}');
  expect(css).not.toContain('.mw-email-body');
  expect(css.split('}').filter(Boolean).every(rule=>rule.startsWith('.mw-quote-body'))).toBe(true);
 });
 it('does not let the rules reach our text when the whole message is sanitised again',async()=>{
  const quote=await quoteForReply({html:HOSTILE,ownedCids:OWNED},'A wrote:');
  const again=await normalizeHtmlSource(`<p>My answer</p>${quote.html}`);
  const css=new DOMParser().parseFromString(again.html,'text/html').querySelector('style')!.textContent!;
  // Every rule still needs the quote container; none can match the message-level wrapper or our paragraph.
  expect(css.split('}').filter(Boolean).every(rule=>/^\.mw-email-body \.mw-quote-body/.test(rule))).toBe(true);
  expect(again.html).not.toMatch(/<blockquote>.*<\/blockquote>.*My answer/s);
  const parsed=new DOMParser().parseFromString(again.html,'text/html');
  expect(parsed.querySelector('blockquote')!.contains([...parsed.querySelectorAll('p')].find(p=>p.textContent==='My answer')??null)).toBe(false);
 });
 it('forward and Outlook variants isolate the original the same way',async()=>{
  for(const quote of [await quoteForForward({html:HOSTILE,ownedCids:OWNED},['Forwarded']),await quoteForOutlook({html:HOSTILE,ownedCids:OWNED},[{label:'From:',value:'A'}])]){
   expect(quote.html).not.toContain('mw-email-body');expect(quote.html).toContain('.mw-quote-body p{color:red');
   expect(quote.html.startsWith('<p></p>')).toBe(true);
  }
 });
 it('the native editor answer block computes to the default look and the quote is not our text',async()=>{
  const quote=await quoteForReply({html:HOSTILE,ownedCids:OWNED},'A wrote:');
  const {api,doc}=mount(quote.html);
  const answer=doc.querySelector('[data-mw-native-answer]') as HTMLElement;
  expect(answer).toBe(doc.querySelector('#mw-native-editor > p'));
  expect(answer.closest('blockquote,.mw-quote-body')).toBeNull();
  const sheets=doc.adoptedStyleSheets.length;expect(sheets).toBeGreaterThan(0);
  // Typing in the answer stays outside the quote and the sent HTML carries only quote-scoped rules.
  answer.textContent='My answer';fireEvent.input(answer);
  const sent=await api.capture();
  const parsed=new DOMParser().parseFromString(sent,'text/html');
  const mine=[...parsed.querySelectorAll('p')].find(p=>p.textContent==='My answer')!;
  expect(mine.closest('blockquote')).toBeNull();expect(mine.getAttribute('style')).toBeNull();expect(mine.hasAttribute('data-mw-native-answer')).toBe(false);
  const css=parsed.querySelector('style')!.textContent!;
  expect(css.split('}').filter(Boolean).every(rule=>rule.startsWith('.mw-quote-body'))).toBe(true);
  expect(parsed.querySelector('blockquote .mw-quote-body')?.getAttribute('dir')).toBe('rtl');
 });
 it('never treats a paragraph of the quote as the answer once ours is deleted',async()=>{
  const quote=await quoteForReply({html:HOSTILE,ownedCids:OWNED},'A wrote:');
  const {doc}=mount(quote.html.replace('<p></p>',''));
  expect(doc.querySelector('[data-mw-native-answer]')?.closest('blockquote')??null).toBeNull();
 });
});

describe('the rich editor reply document',()=>{
 it('keeps the answer paragraph free of marks and outside the blockquote',async()=>{
  const quote=await quoteForReply({html:'<p style="color:#ff0000;text-align:right"><strong>Bold <em>red</em></strong></p>'},'A wrote:');
  const doc=docFromHtml(quote.html);
  expect(doc.firstChild!.type.name).toBe('paragraph');expect(doc.firstChild!.content.size).toBe(0);expect(doc.firstChild!.attrs['align']).toBeNull();
  const state=EditorState.create({doc,selection:TextSelection.create(doc,1)});
  expect(state.storedMarks).toBeNull();expect(state.selection.$from.marks()).toEqual([]);
  const typed=state.apply(state.tr.insertText('mine'));
  const html=htmlFromDoc(typed.doc);
  expect(html.startsWith('<p>mine</p>')).toBe(true);expect(html.indexOf('mine')).toBeLessThan(html.indexOf('<blockquote>'));
 });
});
