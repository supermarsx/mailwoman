/** Bounding rectangles and pointer coordinates are viewport pixels. Fixed
 * controls use layout pixels under CSS zoom; never apply browser/pinch zoom. */
export function interfaceScale(doc:Document=document):number{
 const value=Number(doc.documentElement.style.getPropertyValue('--mw-ui-zoom'));
 return Number.isFinite(value)&&value>=.5&&value<=2?value:1;
}
export function interfacePixels(value:number,doc:Document=document):number{return value/interfaceScale(doc);}

/** Convert a viewport rectangle into the same units as clientWidth/scrollTop. */
export function interfaceRect(rect:Pick<DOMRectReadOnly,'left'|'right'|'top'|'bottom'|'width'|'height'>,doc:Document=document){
 const scale=interfaceScale(doc);
 return {left:rect.left/scale,right:rect.right/scale,top:rect.top/scale,bottom:rect.bottom/scale,width:rect.width/scale,height:rect.height/scale};
}
