export interface EditorCounts {words:number;paragraphs:number;characters:number}
/** Plain text metrics. A document editor supplies its structural paragraph count. */
export function countEditorText(text:string,paragraphs?:number):EditorCounts {
 return {words:(text.match(/[\p{L}\p{N}]+(?:['’-][\p{L}\p{N}]+)*/gu)??[]).length,paragraphs:paragraphs??(text.length?text.split(/\r?\n/).length:0),characters:[...text].length};
}
