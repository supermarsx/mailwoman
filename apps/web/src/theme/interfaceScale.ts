/** Bounding rectangles and pointer coordinates are viewport pixels. Fixed
 * controls use layout pixels under CSS zoom; never apply browser/pinch zoom. */
export function interfaceScale(doc:Document=document):number{
 const value=Number(doc.documentElement.style.getPropertyValue('--mw-ui-zoom'));
 return Number.isFinite(value)&&value>=.5&&value<=2?value:1;
}
export function interfacePixels(value:number,doc:Document=document):number{return value/interfaceScale(doc);}
