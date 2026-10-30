// Initials and palette colour of a generated avatar (t99: split out of
// PersonAvatar.tsx so the account layer, which is in the entry chunk, can use it
// without pulling the sender-icon catalogue (~86 kB of SVG paths) and the avatar
// stylesheets into the first load).
import {AVATAR_COLORS} from './senderIcons/palette.ts';

export function avatarPresentation(name:string,address:string):{initials:string;color:string;palette:number} {
 const label=(name.trim()||address.split('@')[0]||'?').replace(/[\p{Cc}\p{Cf}]/gu,'');
 const words=label.split(/[\s._-]+/u).filter(Boolean);const initials=words.slice(0,2).map(word=>Array.from(word)[0]??'').join('').toLocaleUpperCase().slice(0,4)||'?';
 let hash=2166136261;for(const character of (address.trim()||label).toLowerCase())hash=Math.imul(hash^character.codePointAt(0)!,16777619)>>>0;
 const palette=hash%AVATAR_COLORS.length;
 return {initials,color:AVATAR_COLORS[palette]!,palette};
}
