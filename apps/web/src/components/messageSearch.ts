import type {Email} from '../api/jmap-types.ts';
export function relatedSearchValue(message:Email,field:'from'|'subject'):string|null {
 const value=field==='from'?message.from?.[0]?.email:message.subject;
 if(typeof value!=='string'||!value.trim()||value.length>4096||[...value].some(character=>character.charCodeAt(0)<32||character.charCodeAt(0)===127))return null;
 return value.trim();
}
