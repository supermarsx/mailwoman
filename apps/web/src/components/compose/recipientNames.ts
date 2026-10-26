import {isMailbox,parseRecipients,splitAddressList} from '../../api/jmap.ts';
import type {ContactCard} from '../../api/pim-types.ts';
import {rankSuggestions} from '../../modules/contacts/autocomplete.ts';
/** Resolve only an exact, unique contact name; ambiguous/unknown names remain editable. */
export function checkRecipientNames(value:string,cards:readonly ContactCard[]):{value:string;unresolved:string[];resolved:number}{
 const unresolved:string[]=[];let resolved=0;
 const result=splitAddressList(value).map(entry=>{
  const address=parseRecipients(entry)[0];if(address&&isMailbox(address.email))return entry;
  const matches=rankSuggestions(cards,entry,cards.length).filter(match=>match.name.trim().toLowerCase()===entry.trim().toLowerCase()&&isMailbox(match.email));
  const unique=new Map(matches.map(match=>[match.email.toLowerCase(),match]));
  if(unique.size!==1){unresolved.push(entry);return entry;}
  const match=[...unique.values()][0]!;resolved++;
  return `${JSON.stringify(match.name)} <${match.email}>`;
 });
 return {value:result.join(', '),unresolved,resolved};
}
