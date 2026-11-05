import {inertBody} from './richtext.ts';

/** How far the quoted original reaches after its header: a reply quote is the
 * header paragraph plus the blockquote that follows it; a forward or an
 * Outlook-style quote runs from its header to the end of the body. */
export type QuoteKind='reply'|'tail';

const squash=(value:string)=>value.replace(/\s+/g,' ').trim();

/** The top-level nodes that make up the quote, or null when its header is gone. */
function region(body:HTMLElement,anchor:string,kind:QuoteKind):ChildNode[]|null {
 const wanted=squash(anchor);if(wanted==='')return null;
 const nodes=Array.from(body.childNodes);
 const start=nodes.findIndex(node=>node.nodeType===1&&squash(node.textContent??'').startsWith(wanted));
 if(start<0)return null;
 let from=start;
 if(kind==='tail'&&start>0&&(nodes[start-1] as Element).tagName==='HR')from=start-1;
 if(kind==='tail')return nodes.slice(from);
 const quote=nodes.findIndex((node,index)=>index>start&&(node as Element).tagName==='BLOCKQUOTE');
 return quote<0?nodes.slice(start,start+1):nodes.slice(start,quote+1);
}

/** `current` with its quoted original replaced by the one in `next`, in place:
 * what the user wrote around the quote is kept, and the quote is never added a
 * second time. Null when either side holds no quote (the user removed it). */
export function swapQuote(current:string,next:string,anchor:string,kind:QuoteKind):string|null {
 const body=inertBody(current),fresh=region(inertBody(next),anchor,kind),old=region(body,anchor,kind);
 if(!fresh||!old)return null;
 const first=old[0]!;
 for(const node of fresh)body.insertBefore(body.ownerDocument.importNode(node,true),first);
 for(const node of old)node.remove();
 return body.innerHTML;
}
