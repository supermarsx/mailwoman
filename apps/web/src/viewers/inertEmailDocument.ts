import {parse,defaultTreeAdapter as tree,type DefaultTreeAdapterTypes as T} from 'parse5';

export type InertEmailDocument=Pick<Document,'body'|'head'|'documentElement'|'querySelectorAll'>;
const authoredStyles=new WeakMap<Element,string>();
/** Patch validation must inspect authored URLs before CSSOM can drop invalid CSS. */
export const authoredEmailStyle=(element:Element):string|null=>authoredStyles.get(element)??element.getAttribute('style');
export function setInertAttribute(element:Element,name:string,value:string):void {
 if(name.toLowerCase()==='style'){
  authoredStyles.set(element,value);
  const style=(element as HTMLElement).style;
  if(style)style.cssText=value;
 }else element.setAttribute(name,value);
}
/** Parse without handing untrusted URLs or CSS to an active browser document.
 * Template-owned nodes have no browsing context: no image loads, style-sheet
 * evaluation, custom-element upgrades or inherited-CSP console violations. */
export function inertEmailDocument(source:string):InertEmailDocument {
 const template=document.createElement('template'),owner=template.content.ownerDocument;
 function node(value:T.ChildNode):Node|null {
  if(tree.isTextNode(value))return owner.createTextNode(value.value);
  if(tree.isCommentNode(value))return owner.createComment(value.data);
  if(!tree.isElementNode(value))return null;
  const element=owner.createElementNS(value.namespaceURI,value.tagName);
  for(const attr of value.attrs){
   if(attr.namespace)element.setAttributeNS(attr.namespace,attr.prefix?`${attr.prefix}:${attr.name}`:attr.name,attr.value);
   else setInertAttribute(element,attr.name,attr.value);
  }
  const target=value.tagName==='template'?(element as HTMLTemplateElement).content:element;
  const children=value.tagName==='template'?tree.getTemplateContent(value as T.Template).childNodes:value.childNodes;
  for(const child of children){const converted=node(child);if(converted)target.appendChild(converted);}
  // Sender CSS debug directives are not usable from opaque email documents.
  // Leaving empty/relative map hints makes Firefox DevTools resolve about:srcdoc.
  if(value.tagName==='style')element.textContent=(element.textContent??'').replace(/\/\*[#@]\s*source(?:Mapping)?URL\s*=[\s\S]*?\*\//gi,'');
  return element;
 }
 const parsed=parse(source),html=parsed.childNodes.find(value=>tree.isElementNode(value)&&value.tagName==='html');
 if(!html)throw new Error('Invalid email document');
 const root=node(html) as HTMLElement;
 template.content.append(root);
 const head=root.querySelector('head') as HTMLHeadElement,body=root.querySelector('body') as HTMLBodyElement;
 return {documentElement:root,head,body,querySelectorAll:root.querySelectorAll.bind(root)};
}
