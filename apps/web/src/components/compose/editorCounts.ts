export interface EditorCounts {words:number;paragraphs:number;characters:number}
const utf8=new TextEncoder();
/** Body bytes, including authored markup and data images for HTML callers. */
export function editorBodyBytes(source:string):number {return utf8.encode(source).byteLength;}
/** Plain text metrics. A document editor supplies its structural paragraph count. */
export function countEditorText(text:string,paragraphs?:number):EditorCounts {
 return {words:(text.match(/[\p{L}\p{N}]+(?:['’-][\p{L}\p{N}]+)*/gu)??[]).length,paragraphs:paragraphs??(text.length?text.split(/\r?\n/).length:0),characters:[...text].length};
}
