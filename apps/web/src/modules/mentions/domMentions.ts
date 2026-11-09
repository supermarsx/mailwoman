import {mentionQuery,mentionSuggestions,mergeMentionSuggestions} from './mentions.ts';
import {mentionPopup} from './mentionPopup.ts';
import type {MentionSource} from './mentionSource.ts';
import * as css from './mentionPopup.css.ts';
/** The isolated native editor uses constructed stylesheets, never sender CSS
 * for controls. System colors follow its native light/dark color scheme. */
export const MENTION_FRAME_STYLE=`.${css.popup}{position:fixed;z-index:2000;max-height:16rem;overflow:auto;width:min(22rem,calc(100vw - 2rem));background:Canvas;color:CanvasText;border:1px solid currentColor;padding:4px;margin:0;list-style:none;box-sizing:border-box}.${css.option}{padding:8px;cursor:pointer}.${css.option}[aria-selected=true]{background:Highlight;color:HighlightText}`;
export function attachDomMentions(editor:HTMLElement,source:MentionSource,emit:()=>void):()=>void{
 const doc=editor.ownerDocument;let revision=0,range:Range|null=null,composing=false;
 const picker=mentionPopup(editor,pick=>{if(!range||!editor.contains(range.startContainer)||!editor.contains(range.endContainer))return;revision++;const selection=doc.getSelection();selection?.removeAllRanges();selection?.addRange(range);
  const link=doc.createElement('a');link.href='mailto:'+encodeURIComponent(pick.email);link.textContent='@'+(pick.name.trim()||pick.email);// Avoid HTML string sinks: Trusted Types also guards execCommand('insertHTML').
  // The native editor records this DOM edit through its ordinary emit/history.
  range.deleteContents();const fragment=doc.createDocumentFragment(),space=doc.createTextNode(' ');fragment.append(link,space);range.insertNode(fragment);range.setStartAfter(space);range.collapse(true);selection?.removeAllRanges();selection?.addRange(range);
  range=null;editor.focus();emit();source.onPick(pick);
 },()=>{revision++;range=null;});
 const refresh=():void=>{revision++;const mine=revision;picker.close();range=null;if(composing)return;const selection=doc.getSelection();if(!selection?.isCollapsed||!selection.anchorNode||!editor.contains(selection.anchorNode))return;
  const element=selection.anchorNode.nodeType===1?selection.anchorNode as Element:selection.anchorNode.parentElement;
  if(element?.closest('a,code,pre,blockquote,[data-mw-signature-host],[data-mw-signature]'))return;
  const block=element?.closest('p,div,li,h1,h2,h3,h4,h5,h6')??editor;if(!editor.contains(block))return;
  const before=doc.createRange();before.selectNodeContents(block);before.setEnd(selection.anchorNode,selection.anchorOffset);const text=before.toString(),q=mentionQuery(text);if(!q)return;
  const replace=doc.createRange(),walker=doc.createTreeWalker(block,4);let offset=0,started=false;for(let node=walker.nextNode();node;node=walker.nextNode()){const end=offset+(node.textContent?.length??0);if(!started&&q.from>=offset&&q.from<end){replace.setStart(node,q.from-offset);started=true;}if(started&&q.to<=end){replace.setEnd(node,q.to-offset);break;}offset=end;}if(!started||Array.from(block.querySelectorAll('a,code')).some(link=>replace.intersectsNode(link)))return;range=replace;
  const rect=before.getBoundingClientRect(),scope=source.scope?.(),local=mentionSuggestions(source.cards(),q.query);picker.show(local,rect);
  if(source.lookup&&q.query.trim())void source.lookup(q.query).then(remote=>{if(mine!==revision||scope!==source.scope?.()||!doc.hasFocus())return;picker.show(mergeMentionSuggestions(local,remote),rect);}).catch(()=>undefined);
 };
 const key=(event:KeyboardEvent):void=>{if(picker.key(event))event.stopImmediatePropagation();};const start=():void=>{composing=true;revision++;picker.close();};const end=():void=>{composing=false;refresh();};
 doc.addEventListener('input',refresh);doc.addEventListener('keyup',refreshSelection);doc.addEventListener('keydown',key,true);doc.addEventListener('compositionstart',start);doc.addEventListener('compositionend',end);
 function refreshSelection(event:KeyboardEvent):void{if(['ArrowLeft','ArrowRight','Home','End'].includes(event.key))refresh();}
 return()=>{revision++;picker.destroy();doc.removeEventListener('input',refresh);doc.removeEventListener('keyup',refreshSelection);doc.removeEventListener('keydown',key,true);doc.removeEventListener('compositionstart',start);doc.removeEventListener('compositionend',end);};
}
