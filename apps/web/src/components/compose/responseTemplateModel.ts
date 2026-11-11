export interface ResponseTemplate { id:string; title:string; body:string; subject?:string; to?:string; cc?:string; bcc?:string }
export const TEMPLATE_LIMIT=20;
export const TEMPLATE_HEADER_LIMITS = { subject: 998, to: 2000, cc: 2000, bcc: 2000 } as const;
export const TEMPLATE_HEADERS = ['subject', 'to', 'cc', 'bcc'] as const;
export const TEMPLATE_TITLE_LIMIT=80;
export const TEMPLATE_BODY_LIMIT=8000;
export const TEMPLATE_BYTES_LIMIT=65536;

/** Shape-only reader for an editable draft; incomplete fields remain editable. */
export function templateRows(value:unknown):ResponseTemplate[]{
 if(!Array.isArray(value))return [];
 return value.filter((row):row is ResponseTemplate=>Boolean(row)&&typeof row==='object'&&typeof row.id==='string'&&typeof row.title==='string'&&typeof row.body==='string').slice(0,TEMPLATE_LIMIT);
}
export function validResponseTemplates(value:unknown):value is ResponseTemplate[]{
 if(!Array.isArray(value)||value.length>TEMPLATE_LIMIT)return false;
 const ids=new Set<string>();
 for(const row of value){
  if(!row||typeof row!=='object'||Object.keys(row).some(key=>!['body','id','title',...TEMPLATE_HEADERS].includes(key))||typeof row.id!=='string'||!/^[A-Za-z0-9_-]{1,64}$/.test(row.id)||ids.has(row.id))return false;
  if(typeof row.title!=='string'||!row.title.trim()||[...row.title].length>TEMPLATE_TITLE_LIMIT||hasControl(row.title,false))return false;
  if(typeof row.body!=='string'||!row.body.trim()||[...row.body].length>TEMPLATE_BODY_LIMIT||hasControl(row.body,true))return false;
  for (const field of TEMPLATE_HEADERS) {
   if (!Object.hasOwn(row, field)) continue;
   const value = row[field];
   if (typeof value !== 'string' || [...value].length > TEMPLATE_HEADER_LIMITS[field] || hasControl(value, false)) return false;
  }
  ids.add(row.id);
 }
 return new TextEncoder().encode(JSON.stringify(value)).length<=TEMPLATE_BYTES_LIMIT;
}
function hasControl(text:string,allowWhitespace:boolean):boolean{return [...text].some(char=>{const code=char.codePointAt(0)!;return (code<32||code>=127&&code<=159)&&!(allowWhitespace&&(code===9||code===10||code===13));});}
export function matchingTemplates(rows:readonly ResponseTemplate[],query:string):ResponseTemplate[]{
 const needle=query.trim().toLocaleLowerCase();return rows.filter(row=>`${row.title}\n${row.subject ?? ''}\n${row.body}`.toLocaleLowerCase().includes(needle));
}
